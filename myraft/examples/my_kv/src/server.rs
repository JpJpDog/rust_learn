mod clientpb {
    tonic::include_proto!("clientpb");
}
use anyhow::Result;
use bincode::{deserialize, serialize};
use clientpb::client_rpc_server::{ClientRpc, ClientRpcServer};
use clientpb::{ReadRpcReq, ReadRpcRsp, WriteRpcReq, WriteRpcRsp};
use log::info;
use myraft::network::MyRaftNetwork;
use myraft::raft::MyRaft;
use myraft::storage::MyRaftStorage;
use myraft::{
    async_trait::async_trait, raft::RaftApp, AppData, AppDataResponse, ClientWriteRequest, Raft,
};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::collections::HashMap;
use std::sync::Arc;
use std::u64;
use structopt::StructOpt;
use tokio::{self, spawn};
use tonic::Code;
use tonic::{transport::Server, Request, Response, Status};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadRequest {
    pub key: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadResponse {
    pub data: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WriteRequest {
    Insert { key: u64, value: String },
    Remove { key: u64 },
}

impl AppData for WriteRequest {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WriteResponse {
    Insert { prev: Option<String> },
    Remove { prev: Option<String> },
}

impl AppDataResponse for WriteResponse {}

pub struct KvApp {
    db: Db,
}

#[async_trait]
impl RaftApp for KvApp {
    fn new(sm_path: &str) -> Self {
        Self {
            db: sled::open(sm_path).unwrap(),
        }
    }

    async fn handle_read(&self, req: ReadRequest) -> Result<ReadResponse> {
        let rsp = self.db.get(&serialize(&req.key)?)?;
        let rsp = match rsp {
            Some(rsp) => Some(deserialize(&rsp)?),
            None => None,
        };
        Ok(ReadResponse { data: rsp })
    }

    async fn handle_write(&mut self, req: WriteRequest) -> Result<WriteResponse> {
        match req {
            WriteRequest::Insert { key, value } => {
                let prev = self.db.insert(serialize(&key)?, serialize(&value)?)?;
                self.db.flush_async().await?;
                Ok(WriteResponse::Insert {
                    prev: match prev {
                        Some(prev) => Some(deserialize(&prev)?),
                        None => None,
                    },
                })
            }
            WriteRequest::Remove { key } => {
                let prev = self.db.remove(serialize(&key)?)?;
                self.db.flush_async().await?;
                Ok(WriteResponse::Remove {
                    prev: match prev {
                        Some(prev) => Some(deserialize(&prev)?),
                        None => None,
                    },
                })
            }
        }
    }

    async fn make_snapshot(&self) -> Result<Vec<u8>> {
        let mut map = HashMap::new();
        for kv in self.db.into_iter() {
            let (k, v) = kv?;
            map.insert(k.to_vec(), v.to_vec());
        }
        Ok(serialize(&map)?)
    }

    async fn handle_snapshot(&self, snap: &Vec<u8>) -> Result<()> {
        let map: HashMap<Vec<u8>, Vec<u8>> = deserialize(snap)?;
        for (k, v) in map {
            self.db.insert(k, v)?;
        }
        self.db.flush_async().await?;
        Ok(())
    }

    type ReadReq = ReadRequest;

    type ReadRsp = ReadResponse;

    type WriteReq = WriteRequest;

    type WriteRsp = WriteResponse;
}

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long)]
    id: u64,
    #[structopt(short, long)]
    raft_addr: String,
    // #[structopt(short, long)]
    // cluster_id: u64,
    #[structopt(short, long)]
    client_addr: Option<String>,
    #[structopt(short, long)]
    as_init: bool,
}

type MyKvRaft = Raft<WriteRequest, WriteResponse, MyRaftNetwork<KvApp>, MyRaftStorage<KvApp>>;

struct MyClientRpc {
    core: Arc<MyKvRaft>,
    storage: Arc<MyRaftStorage<KvApp>>,
}

#[async_trait]
impl ClientRpc for MyClientRpc {
    async fn read(&self, request: Request<ReadRpcReq>) -> Result<Response<ReadRpcRsp>, Status> {
        let req = request.into_inner();
        let req = ReadRequest { key: req.id };
        info!("read: {:?}", req);
        match self.storage.handle_read(req).await {
            Ok(rsp) => {
                let rsp = ReadRpcRsp {
                    found: rsp.data.is_some(),
                    data: rsp.data.unwrap_or("".to_string()),
                };
                Ok(Response::new(rsp))
            }
            Err(_) => Err(Status::new(Code::Unknown, "")),
        }
    }

    async fn write(&self, request: Request<WriteRpcReq>) -> Result<Response<WriteRpcRsp>, Status> {
        let req = request.into_inner();
        let req = if req.kind == 0 {
            WriteRequest::Insert {
                key: req.key,
                value: req.data,
            }
        } else {
            WriteRequest::Remove { key: req.key }
        };
        info!("write: {:?}", req);
        match self.core.client_write(ClientWriteRequest::new(req)).await {
            Ok(rsp) => {
                let rsp = rsp.data;
                let rsp = match rsp {
                    WriteResponse::Insert { prev } => WriteRpcRsp {
                        kind: 0,
                        found: prev.is_some(),
                        prev: prev.unwrap_or("".to_string()),
                    },
                    WriteResponse::Remove { prev } => WriteRpcRsp {
                        kind: 1,
                        found: prev.is_some(),
                        prev: prev.unwrap_or("".to_string()),
                    },
                };
                Ok(Response::new(rsp))
            }
            Err(err) => Err(Status::new(
                Code::Unknown,
                format!("call core write error: {}", err),
            )),
        }
    }
}

async fn start_client_service(raft: &MyRaft<KvApp>, client_addr: String) -> Result<()> {
    let client_rpc = MyClientRpc {
        core: raft.my_core.clone(),
        storage: raft.my_storage.clone(),
    };
    let client_addr = client_addr.parse().unwrap();
    info!("listenning client addr: {:?}", client_addr);
    spawn(async move {
        Server::builder()
            .add_service(ClientRpcServer::new(client_rpc))
            .serve(client_addr)
            .await
            .unwrap();
    })
    .await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let opt = Opt::from_args();
    let my_raft = MyRaft::<KvApp>::new(opt.id, opt.raft_addr).await;
    my_raft.join_cluster(1, opt.as_init).await;
    if let Some(client_addr) = opt.client_addr {
        start_client_service(&my_raft, client_addr).await.unwrap();
    }
}
