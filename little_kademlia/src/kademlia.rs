use rand::Rng;
use std::{
    collections::{BinaryHeap, HashSet},
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{
    client_pool::ClientPool,
    error::{JoinError, PingError},
    kadserver::KadServer,
    kbucket::{KBuckets, TestFun},
    msg::{Msg, MsgAck, NearNode},
};

pub type IdType = u16;
pub type SharedKBuckets = Arc<Mutex<KBuckets>>;

const BUCKET_K: usize = 4;
const BUCKET_ALPHA: usize = 2;
const PING_TIMEOUT: Option<Duration> = Some(Duration::from_millis(1000));
const FINDNODE_TIMEOUT: Option<Duration> = Some(Duration::from_millis(1000));

struct Kademlia {
    id: IdType,
    kbuckets: SharedKBuckets,
    client_pool: Arc<ClientPool>,
    _server_handler: JoinHandle<()>,
}

impl Kademlia {
    pub fn new(server_addr: SocketAddr, client_addrs: Vec<SocketAddr>) -> Self {
        let clients = ClientPool::new(client_addrs);
        let clients = Arc::new(clients);
        let clients_clone = clients.clone();
        let id = rand::thread_rng().gen_range(0..std::u16::MAX);
        let kbuckets = Arc::new(Mutex::new(KBuckets::new(
            id,
            BUCKET_K,
            Kademlia::make_kbuckets_test(clients_clone),
        )));
        let kbuckets_clone = kbuckets.clone();
        let handler = thread::spawn(move || {
            let mut server = KadServer::new(id, kbuckets_clone);
            server.routine(server_addr);
        });
        Self {
            client_pool: clients,
            id,
            kbuckets: kbuckets,
            _server_handler: handler,
        }
    }

    fn make_kbuckets_test(clients: Arc<ClientPool>) -> Box<TestFun> {
        let test_fun = move |id: IdType, test_addrs: Vec<SocketAddr>| {
            let msg = Msg::Ping { id };
            let msg = serde_json::to_string(&msg).unwrap();
            let mut sends = Vec::new();
            for addr in test_addrs {
                sends.push((msg.clone(), addr));
            }
            let recvs = clients.send_recv_batch(sends, PING_TIMEOUT);
            let mut rets = Vec::new();
            for recv in recvs {
                if let Some(_recv) = recv {
                    rets.push(true);
                } else {
                    rets.push(false);
                }
            }
            rets
        };
        Box::new(test_fun)
    }

    pub fn ping(&mut self, server_addr: SocketAddr) -> Result<IdType, PingError> {
        let msg = Msg::Ping { id: self.id };
        let msg = serde_json::to_string(&msg).unwrap();
        let ack = self.client_pool.send_recv((msg, server_addr), PING_TIMEOUT);
        if let Some(ack) = ack {
            let ack: Result<MsgAck, _> = serde_json::from_str(&ack);
            if let Ok(ack) = ack {
                if let MsgAck::Ping { id } = ack {
                    let mut guard = self.kbuckets.lock().unwrap();
                    guard.add(vec![(id, server_addr)]);
                    return Ok(id);
                }
            }
        }
        Err(PingError)
    }

    pub fn look_up(&mut self, target: IdType) -> Vec<NearNode> {
        let msg = Msg::FindNode {
            id: self.id,
            target,
        };
        let msg = serde_json::to_string(&msg).unwrap();
        let guard = self.kbuckets.lock().unwrap();
        let mut k_closest_vec = guard.find_k_nearest(target);
        std::mem::drop(guard);
        k_closest_vec.sort();
        let mut finished = false;
        let mut queried = HashSet::new();
        while !finished {
            let mut sends = Vec::new();
            let mut idx = 0;
            for node in &k_closest_vec {
                sends.push((msg.clone(), node.addr));
                queried.insert(node.id);
                idx += 1;
                if idx == BUCKET_ALPHA {
                    break;
                }
            }
            finished = self.update_k_closest(&mut k_closest_vec, sends);
        }
        let mut sends = Vec::new();
        for node in &k_closest_vec {
            if let Some(_) = queried.get(&node.id) {
                sends.push((msg.clone(), node.addr));
            }
        }
        self.update_k_closest(&mut k_closest_vec, sends);
        k_closest_vec
    }

    fn update_k_closest(
        &self,
        k_closest: &mut Vec<NearNode>,
        sends: Vec<(String, SocketAddr)>,
    ) -> bool {
        let mut k_closest_heap = BinaryHeap::new();
        for node in &*k_closest {
            k_closest_heap.push(node.clone());
        }
        let recvs = self.client_pool.send_recv_batch(sends, FINDNODE_TIMEOUT);
        for recv in recvs {
            if let Some(recv) = recv {
                let recv: Result<MsgAck, _> = serde_json::from_str(&recv);
                if let Ok(recv) = recv {
                    if let MsgAck::FindNode { result } = recv {
                        for node in result {
                            k_closest_heap.push(node);
                        }
                    }
                }
            }
        }
        let mut new_k_closest = Vec::new();
        let mut idx = 0;
        while let Some(node) = k_closest_heap.pop() {
            let cur_id = node.id;
            new_k_closest.push(node.clone());
            idx += 1;
            while let Some(peek) = k_closest_heap.peek() {
                if peek.id != cur_id {
                    break;
                } else {
                    k_closest_heap.pop().unwrap();
                }
            }
            if idx == BUCKET_K {
                break;
            }
        }
        let old_len = k_closest.len();
        *k_closest = new_k_closest;
        k_closest.len() != old_len
    }

    pub fn join(&mut self, bootstrap_addr: SocketAddr) -> Result<(), JoinError> {
        let bootstrap_id;
        if let Ok(id) = self.ping(bootstrap_addr) {
            bootstrap_id = id;
        } else {
            return Err(JoinError);
        }
        let mut guard = self.kbuckets.lock().unwrap();
        guard.add(vec![(bootstrap_id, bootstrap_addr)]);
        std::mem::drop(guard);
        let msg = Msg::FindNode {
            id: self.id,
            target: self.id,
        };
        let msg = serde_json::to_string(&msg).unwrap();
        let recv = self
            .client_pool
            .send_recv((msg, bootstrap_addr), FINDNODE_TIMEOUT);
        if let Some(recv) = recv {
            let recv: Result<MsgAck, _> = serde_json::from_str(&recv);
            if let Ok(recv) = recv {
                if let MsgAck::FindNode { result } = recv {
                    let mut adds = Vec::new();
                    for node in result {
                        adds.push((node.id, node.addr));
                    }
                    let mut guard = self.kbuckets.lock().unwrap();
                    guard.add(adds);
                    return Ok(());
                }
            }
        }
        return Err(JoinError);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use std::str::FromStr;
    use std::thread;

    #[test]
    fn test() {
        let mut addrs = Vec::new();
        let mut kads = Vec::new();
        let mut port_n = 12300;
        let kad_n = 16;
        let mut idx = 0;
        while idx < kad_n * 3 {
            addrs.push(SocketAddr::new(
                IpAddr::from_str("127.0.0.1").unwrap(),
                port_n,
            ));
            idx += 1;
            port_n += 1;
        }
        idx = 0;
        while idx < kad_n {
            let kad = Kademlia::new(addrs[idx * 3], vec![addrs[idx * 3 + 1], addrs[idx * 3 + 2]]);
            kads.push(kad);
            idx += 1;
        }

        thread::sleep(Duration::from_millis(500));

        let result1 = kads[0].ping(addrs[3]).unwrap();
        let result2 = kads[1].ping(addrs[0]).unwrap();
        println!("kad2's id is {}, kad1's id is {}", result1, result2);
        for kad in &mut *kads {
            kad.join(addrs[0]).unwrap();
        }
    }
}
