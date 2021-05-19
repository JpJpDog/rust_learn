use crate::thread_pool::ThreadPool;
use rand::Rng;
use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};

const MAX_BUF: usize = 1500;

type RetVal = Arc<(Mutex<Vec<(u32, Option<String>)>>, Condvar)>;

pub struct ClientPool {
    thread_pool: ThreadPool,
    ret_vals: RetVal,
    client_addrs: Vec<SocketAddr>,
}

impl ClientPool {
    pub fn new(client_addrs: Vec<SocketAddr>) -> Self {
        let thread_n = client_addrs.len();
        let thread_pool = ThreadPool::new(thread_n);
        let ret_vals = Arc::new((Mutex::new(Vec::new()), Condvar::new()));
        Self {
            thread_pool,
            client_addrs,
            ret_vals,
        }
    }

    fn private_send(
        &self,
        send_data: String,
        timeout: Option<Duration>,
        server_addr: SocketAddr,
    ) -> u32 {
        let client_addrs_clone = self.client_addrs.clone();
        let ret_clone = self.ret_vals.clone();
        let this_id = rand::thread_rng().gen_range(0..std::u32::MAX);
        let client_routine = move |idx: usize| {
            let mut recv_buf = [0u8; MAX_BUF];
            let client_addr = client_addrs_clone[idx];
            let client = UdpSocket::bind(client_addr).unwrap();
            client.connect(server_addr).unwrap();
            client.set_read_timeout(timeout).unwrap();
            client.send(send_data.as_bytes()).unwrap();
            let (lock, cvar) = &*ret_clone;
            let recv_len = client.recv(&mut recv_buf);
            let mut guard = lock.lock().unwrap();
            if let Ok(recv_len) = recv_len {
                guard.push((
                    this_id,
                    Some(String::from_utf8(recv_buf[0..recv_len].to_vec()).unwrap()),
                ));
            } else {
                guard.push((this_id, None));
            }
            cvar.notify_all();
        };
        self.thread_pool.execute(client_routine);
        this_id
    }

    pub fn send_recv(
        &self,
        send: (String, SocketAddr),
        timeout: Option<Duration>,
    ) -> Option<String> {
        let _recv_id = self.private_send(send.0, timeout, send.1);
        let (lock, cvar) = &*self.ret_vals;
        let mut guard = lock.lock().unwrap();
        while guard.is_empty() {
            guard = cvar.wait(guard).unwrap();
        }
        let (_recv_id1, recv) = guard.pop().unwrap();
        recv
    }

    pub fn send_recv_batch(
        &self,
        sends: Vec<(String, SocketAddr)>,
        timeout: Option<Duration>,
    ) -> Vec<Option<String>> {
        let mut recv_map = HashMap::new();
        let mut rets = vec![None; sends.len()];
        let mut send_cnt = 0;
        for (msg, addr) in sends {
            let recv_id = self.private_send(msg, timeout.clone(), addr);
            recv_map.insert(recv_id, send_cnt);
            send_cnt += 1;
        }
        let mut recv_cnt = 0;
        let (lock, cvar) = &*self.ret_vals;
        let mut guard = lock.lock().unwrap();
        while recv_cnt < send_cnt {
            if guard.is_empty() {
                guard = cvar.wait(guard).unwrap();
            }
            let (recv_id1, recv) = guard.pop().unwrap();
            let idx = recv_map.get(&recv_id1);
            if let Some(idx) = idx {
                rets[*idx] = recv;
            }
            recv_cnt += 1;
        }
        rets
    }
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;
    use std::str::FromStr;
    use std::thread;

    use super::*;

    #[test]
    fn test_sends() {
        let server_addr1 = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 12201);
        let server_addr2 = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 12202);
        let client_addr1 = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 12203);
        let client_addr2 = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 12204);
        let server_routine = move |server_addr: SocketAddr| {
            let server = UdpSocket::bind(server_addr).unwrap();
            let mut recv_buf = [0u8; MAX_BUF];
            loop {
                let (_recv_len, client_addr) = server.recv_from(&mut recv_buf).unwrap();
                server.send_to(&mut recv_buf, client_addr).unwrap();
            }
        };
        let _handler1 = thread::spawn(move || server_routine(server_addr1));
        let _handler2 = thread::spawn(move || server_routine(server_addr2));
        thread::sleep(Duration::from_millis(200));

        let clients = ClientPool::new(vec![client_addr1, client_addr2]);
        let send_msg = String::from("hello world!");
        let sends = vec![
            (send_msg.clone(), server_addr1.clone()),
            (send_msg.clone(), server_addr2),
        ];

        let recv = clients.send_recv((send_msg, server_addr1), None);
        if let Some(recv) = recv {
            println!("recv: {}", recv);
        } else {
            println!("not recv!");
        }

        let recvs = clients.send_recv_batch(sends, None);
        for recv in recvs {
            if let Some(recv) = recv {
                println!("recv: {}", recv);
            } else {
                println!("not recv!");
            }
        }
    }
}
