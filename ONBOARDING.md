# GnosisVPN PoC User Onboarding

## Placeholder/to be set up

- `CRYPTPAD_ONBOARDING_FORM`
- `GNOSISVPN_ENDPOINTS_WEBSITE`

## Step by Step Guide

1. Download service binary from [Github release](https://github.com/hoprnet/gnosis-vpn-client-system-service/releases).

2. Start service binary with privileged access: `sudo ./gnosis-vpn`.

3. Once started without any configuration the service will generate a wireguard priv pub keypair to use.
   Look for `****** Generated wireguard private key ******` and `****** Use this pub_key for onboarding ****** public_key=<pubkey>`.
   Copy `<pubkey>` and provide it to `CRYPTPAD_ONBOARDING_FORM`.
   Also provide a one off drop location to receive your assigned device IP.
   E.g. use [rlim](https://rlim.com/) to create a custom url and paste that url alongside with your edit code into the cryptpad form field.

4. After someone picked up your public key and added it to our session servers you will get your device IP back via your drop location.

5. Open `/etc/gnosisvpn/config.toml` in edit mode and provide your entry node credentials.

Uncomment `entryNode` section and adjust values as needed:

```toml
[entryNode]
# URL pointing to API access of entry node
endpoint = "<your node API url>"
# API access token
api_token = "<your node API token>"
```

6. Still in edit mode enter your assigned IP in the wireguard section:

```toml
[wireguard]
# local interface IP, onboarding info will provide this
address = "<the IP y ou got in your drop off location>"
```

7. Visit `GNOSISVPN_ENDPOINTS_WEBSITE` and choose an exit location
   Update parameters in `/etc/gnosisvpn/config.toml`:

```toml
[wireguard]
# public server peer id - onboarding info will provide this
serverPublicKey = "<exit location server public key>"

[session]
# exit node peer id - the node where the session should terminate
destination = "<exit location peer id>"

[session.target]
# host of the session endpoint without schema
host = "<exit location wg host>"
# port of the session endpoint
port = <exit location wg port>
```

8. At this point the you might see some notificaiton that a `wg0-gnosisvpn` interface is now connected.
   The hoprd session was opened by the service and will kept open.
   Wireguard is also connected and you will be able to use a socks5 proxy on your device.
