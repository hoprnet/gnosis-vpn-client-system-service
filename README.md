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

## PoC wireguard handling

The service aims to handle wireguard session setup.

- need config file in `/etc/gnosisvpn.yaml`

```yaml
_DEBUG_:
    Session:
        Target:
            Type: Plain | Sealed (default: Plain)
            Endpoint: <URL> (default: wireguard.staging.hoprnet.link:51820)
        Capabilities: (default: Segmentation)
            - Segmentation
            - Retransmission

Wireguard:
    PrivateKeyFile: <Path> (default: <empty>)
```

Wireguard handling for linux:

These scenarios are considered:

1.

- after startup determine that `wg` and `wg-quick` are available
- report error otherwise and skip wireguard handling - gnosisvpn-ctl status request shows disabled wireguard handling

2a. wg tools available

- check if gnovpn wireguard configuration is available in `/etc/wireguard/`
- search reserved namespace: `wg0-gnosisvpn` in `/etc/wireguard/` for wg config file

2b. no wg tools

- wait for session parameters via gnosisvpn-ctl and skip 3

3a. wg conf found

- wait for session parameters via gnosisvpn-ctl
- ensure that endpoint in config is set correctly
- adjust and restart device (if necessary)

3b. wg conf not found - private wg key given

- wait for session parameters via gnosisvpn-ctl
- generate wg config file and start device

3c. wg config not found - no private wg key given

- generate new private key
- report public key on gnosisvpn-ctl status request with instruction to send public key to wg admin
- wait for session parameters via gnosisvpn-ctl
- generate wg config file and start device
- if wg has connection problems: report public key on gnosisvpn-ctl status request with instruction to send public key to wg admin

4a. wg tools available

- monitor session as usual
- monitor wg output to report potential connection problems

4b. no wg tools

- monitor session as usual

The service will have to store local data and generated secrets to make this work better.
Essentially local data will be put under the home folder of the service user - once installer setup does prepare this.
For now we could put this data in `/opt/.gnosisvpn`.
