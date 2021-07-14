```shell
RUST_LOG=info cargo run --bin raft_server -- --id=1 --raft-addr=127.0.0.1:11111 --client-addr=127.0.0.1:11112 --as-init=true
RUST_LOG=info cargo run --bin raft_server -- --id=2 --raft-addr=127.0.0.1:22222 --client-addr=127.0.0.1:22223
RUST_LOG=info cargo run --bin raft_server -- --id=3 --raft-addr=127.0.0.1:33333 --client-addr=127.0.0.1:33334
cargo run --bin raft_client -- --client-addr=http://127.0.0.1:11112
```