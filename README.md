# Gnosis VPN Client System Service

## Development usage

`$ cargo build`

Start system service:

`sudo RUST_LOG=debug ./target/debug/gnosis-vpn`

Send commands from control application:

`RUST_LOG=info cargo run --bin gnosis-vpn-ctl -- entry-node --endpoint http://127.0.0.1:19091 --api-token ^^LOCAL-testing-123^^ --listen-host "ip://0.0.0.0:60006" exit-node --peer-id 12D3KooWDsMBB9BiK8zg4ZbA6cgNFpAWikTyyYPKqcNHDaq8samm`

Get state of the service

`RUST_LOG=info cargo run --bin gnosis-vpn-ctl -- status`
