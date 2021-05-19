use std::{cmp::Ordering, net::SocketAddr};

use serde::{Deserialize, Serialize};

use crate::kademlia::IdType;

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct NearNode {
    pub dist: IdType,
    pub id: IdType,
    pub addr: SocketAddr,
}

impl Ord for NearNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.dist.cmp(&self.dist)
    }
}

impl PartialOrd for NearNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
pub enum Msg {
    Ping { id: IdType },
    FindNode { id: IdType, target: IdType },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
pub enum MsgAck {
    Ping { id: IdType },
    FindNode { result: Vec<NearNode> },
}
