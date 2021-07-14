pub mod network;
pub mod raft;
pub mod storage;
mod raftpb {
    tonic::include_proto!("raftpb");
}

// pub use crate::network::MyRaftNetwork;
// pub use crate::raft::MyRaft;
// pub use crate::storage::MyRaftStorage;
pub use async_raft::async_trait;
pub use async_raft::raft::ClientWriteRequest;
pub use async_raft::{AppData, AppDataResponse, Raft};
