use std::net::{SocketAddr, UdpSocket};

use crate::{
    kademlia::{IdType, SharedKBuckets},
    msg::{Msg, MsgAck},
};

const MAX_BUF: usize = 1500;

pub struct KadServer {
    id: IdType,
    kbuckets: SharedKBuckets,
}

impl KadServer {
    pub fn new(id: IdType, kbucktes: SharedKBuckets) -> Self {
        KadServer {
            id,
            kbuckets: kbucktes,
        }
    }

    pub fn routine(&mut self, server_addr: SocketAddr) {
        let server = UdpSocket::bind(server_addr).unwrap();
        let mut recv_buf = [0u8; MAX_BUF];
        loop {
            let (recv_len, client_addr) = &server.recv_from(&mut recv_buf).unwrap();
            let msg: Msg =
                serde_json::from_str(std::str::from_utf8(&recv_buf[0..*recv_len]).unwrap())
                    .unwrap();
            let ack;
            let from_id;
            match msg {
                Msg::Ping { id } => {
                    ack = Some(self.ping_back());
                    from_id = id;
                }
                Msg::FindNode { id, target } => {
                    ack = Some(self.find_node_back(target));
                    from_id = id;
                }
            }
            let mut guard = self.kbuckets.lock().unwrap();
            guard.add(vec![(from_id, *client_addr)]);
            if let Some(ack) = ack {
                server.send_to(ack.as_bytes(), client_addr).unwrap();
            }
        }
    }

    fn ping_back(&self) -> String {
        let ack = MsgAck::Ping { id: self.id };
        serde_json::to_string(&ack).unwrap()
    }

    fn find_node_back(&self, target: IdType) -> String {
        let guard = self.kbuckets.lock().unwrap();
        let k_closest = guard.find_k_nearest(target);
        let ack = MsgAck::FindNode { result: k_closest };
        serde_json::to_string(&ack).unwrap()
    }
}
