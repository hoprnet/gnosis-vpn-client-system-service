# Gnosis VPN Client System Service

## Development usage

Start system service:

`RUST_LOG=info cargo run --bin gnosis-vpn -- --socket $(pwd)/gnovpn.sock`

Send commands from control application:

`RUST_LOG=info cargo run --bin gnosis-vpn-ctl -- --socket $(pwd)/gnovpn.sock --help`

## Testing usage

The service will open the hopr session. It also periodically monitors the session to reopen it, if necessary.

Replace step 9b from the [testing gist](https://gist.github.com/NumberFour8/8cc7bf88d0a30fbbe0e94a6fec2f1077):

9b. Start the VPN service:

`RUST_LOG=info cargo run --bin gnosis-vpn -- --socket $(pwd)/gnovpn.sock`

9c. From another shell run commands via the control application:

```bash
RUST_LOG=info cargo run --bin gnosis-vpn-ctl -- --socket $(pwd)/gnovpn.sock entry-node --endpoint http://127.0.0.1:19091 --api-token ^^LOCAL-testing-123^^ exit-node --peer-id 12D3KooWFYU4hNaHtpxcyodHLoskd2Cw6irx8wsdjw1cFEgiDSg3
```
