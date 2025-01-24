# Onboarding

Setting up the GnosisVPN PoC can be somewhat complex, as it involves multiple steps and configuration details:

- **Download the binary** and run it with several env var parameters.
- **Manually prepare** and configure a WireGuard interface on top of your GnosisVPN session.
- **Configure the GnosisVPN service** using information from three separate sources:
  1. Your entry node credentials
  2. Your assigned device IP
  3. Your chosen exit location

Please select your operating system to begin:

- [Instructions for MacOS](#instructions-for-macos)
- [Instructions for Linux](#instructions-for-linux)

---

## Instructions for MacOS

### 1. Download the latest binary [MacOS]

Download the latest service binary for your system by visiting the [GitHub releases](https://github.com/hoprnet/gnosis-vpn-client-system-service/releases) page.
Choose the binary that matches your system:

| System                | Binary                      |
| --------------------- | --------------------------- |
| macOS with ARM chip   | `gnosis-vpn-aarch64-darwin` |
| macOS with Intel chip | `gnosis-vpn-x86_64-darwin`  |

### 2. Generate Wireguard public key [MacOS]

1. Download the [WireGuard app](https://apps.apple.com/us/app/wireguard/id1451685025) from the Mac App Store.
2. Launch WireGuard, create an **Empty tunnel**, name it, and save. Copy the public key of the newly created tunnel.

### 3. Prepare secure input to receive assigned device IP [MacOS]

Create a secure input location where you will receive your assigned device IP.

1. Visit [rlim.com](https://rlim.com/).
2. Enter "Custom url" input field and provide some input (e.g.: `toms-feedback-gvpn`).
3. Copy url from browser address bar (e.g.: `https://rlim.com/toms-feedback-gvpn`).
4. Copy edit code from top line.

### 4. Provide necessary data to be eligible for GnosisVPN PoC demo [MacOS]

Provide your public key, the **rlim.com** URL, and the edit code in our [onboarding form](https://cryptpad.fr/form/#/2/form/view/bigkDtjj+9G3S4DWCHPTOjfL70MJXdEWTDjkZRrUH9Y/).
If you have trouble opening cryptpad, please try to open it in incognito mode.

### 5. Wait until you get notified about receiving device IP [MacOS]

After someone picked up your public key and added it to our session servers you will get your device IP back via your **rlim.com** document.

### 6. Configure Gnosis VPN service configuration - hoprd entry node [MacOS]

1. Create a copy of [sample config](./sample.config.toml) and move it to your desired location. In this guide we will just assume that you named it `config.toml`.
2. Open `config.toml` in edit mode and uncomment `entryNode` section to adjust values as needed:

```toml
# this section is used to inform about your hoprd entry node
[entryNode]
# URL pointing to API access of entry node with schema and port (e.g.: `http://123.456.7.89:3002`)
endpoint = "<entry node API endpoint>"
# API access token
apiToken = "<entry node API token>"
```

### 7. Configure Gnosis VPN service configuration - gnosisvpn exit location [MacOS]

Visit `GNOSISVPN_ENDPOINTS_WEBSITE` and choose an exit location.
Copy the exit node configuration into your `config.toml` or update parameters manually (after uncommenting) like this:

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

### 8. Configure Gnosis VPN service configuration - static port configuration [MacOS]

You can configure a session to run on a static port on your entry node. This is useful if you set up a firewall rule to allow traffic on specific ports only.
Go back to the `[session]` section and have a look at the optional `listenHost` parameter.
Uncomment it like shown in this example to provide your static port.

```toml
[session]

...

# [OPTIONAL] listen host - specify internal listen host on entry node
# if you have a firewall running and can only use static ports you need to adjust this setting
# in general if you want to establish a session on specific port, just provide this port here with a leading `:` (e.g.: `:60006`)
listenHost = ":60006"
```

### 9. Ready to start the service binary [MacOS]

1. Return to your downloaded binary and make it executable:

```bash
# <system> matches the one you chose earlier
chmod +x ./gnosis-vpn-<system>
```

2. Provide the path to your configuration file and a socket path to start the service binary.
   If you do not want to provide that socket path, you can also start the binary with privileged access and it will use `/var/run/gnosisvpn.sock` as it's socket.

```bash
# <system> matches the one you chose earlier
GNOSISVPN_CONFIG_PATH=./config.toml GNOSISVPN_SOCKET_PATH=./gnosisvpn.sock ./gnosis-vpn-<system>

# or with privileged access
GNOSISVPN_CONFIG_PATH=./config.toml sudo ./gnosis-vpn-<system>`
```

3. Because of macOS security settings, you may see a message that says “macOS cannot verify that this app is free from malware.”
   Click "Cancel", then open System Settings → Privacy & Security, scroll down to Security, and find the blocked binary. Click "Allow Anyway".

4. In your terminal, run the command to start the binary again. MacOS will prompt you one more time to confirm if you want to open it. Click "Open".

If you see immediate errors on startup it is most likely due to errors in your configuration settings.
The binary should tell you which setting parameter might be wrong.

### 10. Edit the newly created WireGuard tunnel [MacOS]

In the WireGuard app, edit the tunnel you created:

```conf
[Interface]
PrivateKey = <Generated automatic by WireGuard app>
Address = <device IP - received via drop location, e.g.: 20.0.0.5/32>

[Peer]
PublicKey = <wg server pub key - listed on GNOSISVPN_ENDPOINTS_WEBSITE>
Endpoint = <entry node IP:60006 - the port needs to match your listenHost configuraiton>
AllowedIPs = <what traffic do you want to route - usually the base of device IP would be a good start, e.g.: 20.0.0.0/24, set to 0.0.0.0/0 to route all traffic>
PersistentKeepalive = 30
```

---

## Instructions for Linux

### 1. Download the latest binary [Linux]

Download the latest service binary for your system by visiting the [GitHub releases](https://github.com/hoprnet/gnosis-vpn-client-system-service/releases) page.
Choose the binary that matches your system:

| system                    | binary                     |
| ------------------------- | -------------------------- |
| linux with x86 chip       | `gnosis-vpn-x86_64-linux`  |
| linux with newer ARM chip | `gnosis-vpn-aarch64-linux` |
| linux with older ARM chip | `gnosis-vpn-armv7l-linux`  |

For now just download it and keep it ready.

### 2. Generate WireGuard keypair [Linux]

Follow guidelines on official [WireGuard documentation](https://www.wireguard.com/quickstart/#key-generation).
Usually:

```bash
wg genkey | tee privatekey | wg pubkey > publickey
```

### 3. Prepare secure input to receive assigned device IP [Linux]

Create a secure input location where you will receive your assigned device IP.

1. Visit [rlim.com](https://rlim.com/).
2. Enter "Custom url" input field and provide some input (e.g.: `toms-feedback-gvpn`).
3. Copy url from browser address bar (e.g.: `https://rlim.com/toms-feedback-gvpn`).
4. Copy edit code from top line.

### 4. Provide necessary data to be eligible for GnosisVPN PoC demo [Linux]

1. Preview public key:

```bash
cat publickey | xclip -r -sel clip
```

2. Provide your public key, the **rlim.com** URL, and the edit code in our [onboarding form](https://cryptpad.fr/form/#/2/form/view/bigkDtjj+9G3S4DWCHPTOjfL70MJXdEWTDjkZRrUH9Y/).
   If you have trouble opening cryptpad, please try to open it in incognito mode.

### 5. Wait until you get notified about receiving device IP [Linux]

After someone picked up your public key and added it to our session servers you will get your device IP back via your **rlim.com** document.

### 6. Configure Gnosis VPN service configuration - hoprd entry node [Linux]

1. Copy [sample config](./sample.config.toml) to `/etc/gnosisvpn/config.toml` and open it in edit mode.
   If you don't like the configuration file location you can override this default via `GNOSISVPN_CONFIG_PATH` env var.

2. Uncomment `entryNode` section and adjust values as needed:

```toml
# this section is used to inform about your hoprd entry node
[entryNode]
# URL pointing to API access of entry node with schema and port (e.g.: `http://123.456.7.89:3002`)
endpoint = "<entry node API endpoint>"
# API access token
apiToken = "<entry node API token>"
```

### 7. Configure Gnosis VPN service configuration - gnosisvpn exit location [Linux]

Visit `GNOSISVPN_ENDPOINTS_WEBSITE` and choose an exit location. Update parameters in `/etc/gnosisvpn/config.toml`:

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

### 8. Configure Gnosis VPN service configuration - static port configuration [Linux]

You can configure a session to run on a static port on your entry node. This is useful if you set up a firewall rule to allow traffic on specific ports only.
Go back to the `[session]` section and have a look at the optional `listenHost` parameter.
Uncomment it like shown in this example to provide your static port.

```toml
[session]

...

# [OPTIONAL] listen host - specify internal listen host on entry node
# if you have a firewall running and can only use static ports you need to adjust this setting
# in general if you want to establish a session on specific port, just provide this port here with a leading `:` (e.g.: `:60006`)
listenHost = ":60006"
```

### 9. Ready to start the service binary [Linux]

1. Return to your downloaded binary and make it executable:

```bash
# <system> matches the one you chose earlier
chmod +x ./gnosis-vpn-<system>
```

2. Start service binary

```bash
# <system> matches the one you chose earlier
# with privileged access
sudo ./gnosis-vpn-<system>`
# without privileged access
GNOSISVPN_CONFIG_PATH=./config.toml GNOSISVPN_SOCKET_PATH=./gnosisvpn.sock ./gnosis-vpn-<system>
```

If you see immediate errors on startup it is most likely due to errors in your configuration settings.
The binary should tell you which setting parameter might be wrong.

### 10. Create a wireguard interface and connect to the created session [Linux]

Create a file called `wg-gnosisvpn-beta.conf` inside `/etc/wireguard/` with the following content:

```conf
[Interface]
PrivateKey = <content privatekey>
Address = <device IP - received via drop location, e.g.: 20.0.0.5/32>

[Peer]
PublicKey = <wg server pub key - listed on GNOSISVPN_ENDPOINTS_WEBSITE>
Endpoint = <entry node IP:60006 - the port needs to match your listenHost configuraiton>
AllowedIPs = <what traffic do you want to route - usually the base of device IP would be a good start, e.g.: 20.0.0.0/24, set to 0.0.0.0/0 to route all traffic>
PersistentKeepalive = 30
```

### 11. Start up wireguard [Linux]

Start up wireguard with `wg-quick up wg-gnosisvpn-beta`.

## [OPTIONAL][EXPERIMENTAL] Let GnosisVPN handle WireGuard connection

**NOTE:** This is an experimental feature and only available on Linux.

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
