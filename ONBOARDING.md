# GnosisVPN PoC User Onboarding

## Placeholder/to be set up

- `CRYPTPAD_ONBOARDING_FORM`
- `GNOSISVPN_ENDPOINTS_WEBSITE`

## Outline

Setting up GnosisVPN PoC is quite complicated and requires multiple steps and configuration input sources.

In general:

- download binary and get it to work with privileged access
- manually prepare and set up wireguard interface that can run on top of GnosisVPN session
- GnosisVPN service configuration will be configured with info from three separate sources:
  - your entry node credentials
  - your assigned device IP
  - your exit location choice

## Step by Step Guide

1. Download the latest service binary for your system.
   Visit [Github release](https://github.com/hoprnet/gnosis-vpn-client-system-service/releases) page and choose depending on your system:

| system                    | binary                      |
| ------------------------- | --------------------------- |
| macOS with ARM chip       | `gnosis-vpn-aarch64-darwin` |
| macOS with Intel chip     | `gnosis-vpn-x86_64-darwin`  |
| linux with x86 chip       | `gnosis-vpn-x86_64-linux`   |
| linux with newer ARM chip | `gnosis-vpn-aarch64-linux`  |
| linux with older ARM chip | `gnosis-vpn-armv7l-linux`   |

For now just download it and keep it ready.

2. Generate WireGuard keypair:

Follow guidelines on official [WireGuard documentation](https://www.wireguard.com/quickstart/#key-generation):

```bash
# linux only
wg genkey | tee privatekey | wg pubkey > publickey
```

On macOS use the Wireguard GUI app to generate a key pair.

3. Create a secure input location where you will receive your assigned device IP.
   We recommend using [rlim](https://rlim.com/) to create an editable drop location.
   See [Create a one off drop location using rlim](#create-a-one-off-drop-location-using-rlim).

4. Provide your public key, rlim url and edit code to `CRYPTPAD_ONBOARDING_FORM`.

```bash
# linux only: copy public key to clipboard
cat publickey | xclip -r -sel clip
# macOS only:
cat publickey | pbcopy
```

On macOS you can also manually copy the public key from the GUI app.

Paste the public key into the form field.
Additionally paste the custom url you created with rlim and the edit code into the cryptpad form field.
Optionally provide some other means of communication if you want to ensure we have a channel to reach out to you.

5. After someone picked up your public key and added it to our session servers you will get your device IP back via your drop location.

6. Configure Gnosis VPN service configuration - hoprd entry node

Copy [sample config](./sample.config.toml) to `/etc/gnosisvpn/config.toml` and open it in edit mode.

Uncomment `entryNode` section and adjust values as needed:

```toml
# this section is used to inform about your hoprd entry node
[entryNode]
# URL pointing to API access of entry node with schema and port (e.g.: `http://123.456.7.89:3002`)
endpoint = "<entry node API endpoint>"
# API access token
apiToken = "<entry node API token>"
```

7. Configure Gnosis VPN service configuration - gnosisvpn exit location

Visit `GNOSISVPN_ENDPOINTS_WEBSITE` and choose an exit location.
Update parameters in `/etc/gnosisvpn/config.toml`:

```toml
# this section holds exit location information and transport parameters
[session]
# the exit node peer id where the session should terminate
destination = "<exit node peer id>"

# this section holds the target information of the session
[session.target]
# host of the session endpoint without schema
host = "<exit location wg host>"
# port of the session endpoint
port = <exit location wg port>
```

8. Configure Gnosis VPN service configuration - static port configuration

You can configure a session to run on a static port on your entry node.
This is useful if you set up a firewall rule to allow traffic on specific ports only.
Go back to the `[session]` section and have a look at the optional `listenHost` parameter.

```toml
[session]

...

# [OPTIONAL] listen host - specify internal listen host on entry node
# if you have a firewall running and can only use static ports you need to adjust this setting
# in general if you want to establish a session on specific port, just provide this port here with a leading `:` (e.g.: `:60006`)
listenHost = ":60006"
```

9. Ready to start the service binary.

Go back to your downloaded binary.
Make it executable:

```bash
# <system> matches the one you chose earlier
chmod +x ./gnosis-vpn-<system>
```

Start it with privileged access:

```bash
# <system> matches the one you chose earlier
sudo ./gnosis-vpn-<system>`
```

At this point macOS might not allow you to execute this binary due to security settings.
If so, go to settings, privacy & security and click allow anyway.

You should be able to start the binary now.

If you see immediate errors on startup it is most likely due to errors in your configuration settings.
The binary should tell you which setting parameter might be wrong.

10. Create a wireguard interface and connect to the created session:

Create a file called `wg-gnosisvpn-beta.conf` inside `/etc/wireguard/` with the following content:

```conf
[Interface]
PrivateKey = <content privatekey>
Address = <device IP - received via drop location>

[Peer]
PublicKey = <wg server pub key - listed on GNOSISVPN_ENDPOINTS_WEBSITE>
Endpoint = <entry node IP:60006 - the port needs to match your listenHost configuraiton>
AllowedIPs = <what traffic do you want to route - usually the base of device IP would be a good start, e.g.: 10.34.0.0/24, set to 0.0.0.0/0 to route all traffic>
```

11. Start up wireguard with `wg-quick up wg-gnosisvpn-beta`.

## [OPTIONAL] Let GnosisVPN handle WireGuard connection

NOTE: This is an experimental feature and only available on linux.

Instead of using wireguard to generate your key pair, make sure wg-tools are installed and available on your system.
Immediately after step 1 start the service as outlined in step 9.
Skip step 2.
Once started without any configuration the service will generate a wireguard priv pub keypair to use.

Look for `****** Generated wireguard private key ******` and `****** Use this pub_key for onboarding ****** public_key=<pubkey>`.
Copy `<pubkey>` and provide it in step 3.

Instead of setting up wireguard manually in step 10 provide the configuration inside `/etc/gnosisvpn/config.toml`:

```toml
# this section holds the wireguard specific settings
[wireguard]
# local interface IP, onboarding info will provide this
address = "10.34.0.8/32"
# wireguard server public peer id - onboarding info will provide this
serverPublicKey = "<wg server public peer id>"
```

At this point the you might see some notificaiton that a `wg0-gnosisvpn` interface is now connected.
The hoprd session was opened by the service and will kept open.
Wireguard is also connected and you will be able to use a socks5 proxy on your device.

## Create a one off drop location using rlim

1. Visit [rlim](https://rlim.com/).
2. Enter "Custom url" input field and provide some input (e.g.: `toms-feedback-gvpn`).
3. Copy url from browser address bar (e.g.: `https://rlim.com/toms-feedback-gvpn`).
4. Copy edit code from top line.
5. Provide both the url and edit code to `CRYPTPAD_ONBOARDING_FORM`.
