# Gnosis VPN Client System Service

The service establishes a VPN connection to a remote endpoint.
It handles hoprd session and optinally wireguard setup (linux only).

## General usage

The service needs to run as root in order to setup the VPN connection.
It loads all parameters from it's configuration file.
This is located in:

```sh
/etc/gnosisvpn/config.toml
```

Copy [./sample.config.toml](sample config) to `/etc/gnosisvpn/config.toml` and adjust the values to your needs.

Start the service with privileged access:

```sh
sudo ./gnosis-vpn-<system-arch>
```

## Onboarding

On MacOS follow [ONBOARDING MACOS](./ONBOARDING-MACOS.md) for a detailed onboarding process ([ONBOARDING LINUX](./ONBOARDING-LINUX.md) on Linux).

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
