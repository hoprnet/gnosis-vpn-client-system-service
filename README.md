# Gnosis VPN Client System Service

The service currently opens a hoprd session and monitors that session.
It will reopen that session if it goes down.

## General usage

In order to work ther service needs to be started in the background.
Run `sudo gnosis-vpn` to start the service.
Open a separate shell to interact with the service.
Run `gnosis-vpn-ctl --help` to see available commands.

A session needs to know the entry and the exit node on which it should connect.
Here is a sample command to open a session on a local cluster:

```
gnosis-vpn-ctl \
    entry-node \
        --endpoint http://127.0.0.1:19091 \
        --api-token ^^LOCAL-testing-123^^ \
        --listen-host ":60006" \
    exit-node \
        --peer-id 12D3KooWKjT35UopfVhGHa8dxGZ6ds8r4rfQdd9rw4PiQZp8HXiA
```

## Development usage

`cargo build`

Start system service:

`sudo RUST_LOG=debug ./target/debug/gnosis-vpn`

Send commands from control application:

`RUST_LOG=info cargo run --bin gnosis-vpn-ctl -- entry-node --endpoint http://127.0.0.1:19091 --api-token ^^LOCAL-testing-123^^ --listen-host "ip://0.0.0.0:60006" exit-node --peer-id 12D3KooWDsMBB9BiK8zg4ZbA6cgNFpAWikTyyYPKqcNHDaq8samm`

Get state of the service

`RUST_LOG=info cargo run --bin gnosis-vpn-ctl -- status`

## Deployment

Show potential deployment targets:

`nix flake show`

Build for a target, e.g. `x86_64-linux`:

`nix build .#gnosisvpn-x86_64-linux`

The resulting binaries are in `results/bin/`:

```
$ ls -l result*/bin/
result/bin/:
total 4752
-r-xr-xr-x 1 root root 4863368 Jan  1  1970 gnosis-vpn
-r-xr-xr-x 1 root root 1740048 Jan  1  1970 gnosis-vpn-ctl
```
