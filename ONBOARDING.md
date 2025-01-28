# Onboarding

Setting up the GnosisVPN PoC can be somewhat complex, as it involves multiple steps and configuration details:

- **Download the binary file** and run it with several env var parameters.
- **Manually prepare** and configure a WireGuard interface on top of your GnosisVPN connection.
- **Configure the GnosisVPN service** using information from three separate sources:
  1. Your hoprd node credentials
  2. Your assigned device IP
  3. Your chosen exit location

Please select your operating system to begin:

- [Instructions for MacOS](#instructions-for-macos)
- [Instructions for Linux](#instructions-for-linux)

---

## Instructions for MacOS

### 1. Download the latest binary file [MacOS]

Download the latest GnosisVPN binary file for your system by visiting the [GitHub releases](https://github.com/hoprnet/gnosis-vpn-client-system-service/releases) page.
Choose the binary file that matches your system:

| System                | Binary file                      |
| --------------------- | --------------------------- |
| macOS with ARM chip   | `gnosis-vpn-aarch64-darwin` |
| macOS with Intel chip | `gnosis-vpn-x86_64-darwin`  |

### 2. Generate Wireguard public key [MacOS]

1. Download the [WireGuard app](https://apps.apple.com/us/app/wireguard/id1451685025) from the Mac App Store.
2. Launch WireGuard, create an **Empty tunnel**, name it, and save. Copy the public key of the newly created tunnel.

### 3. Prepare secure input to receive assigned device IP [MacOS]

Create a secure input location where you will receive your assigned device IP.

1. Go to rlim.com.
2. Locate the "Custom URL" input field and enter your desired text (e.g., `toms-feedback-gvpn`).
3. Save the generated URL from the browser's address bar (e.g., `https://rlim.com/toms-feedback-gvpn`).
4. Note the edit code at the top for the next step.

### 4. Provide necessary data to be eligible for GnosisVPN PoC demo [MacOS]

Provide your public key, the **rlim.com** URL, and the edit code in our [onboarding form](https://cryptpad.fr/form/#/2/form/view/bigkDtjj+9G3S4DWCHPTOjfL70MJXdEWTDjkZRrUH9Y/).
If you have trouble opening cryptpad, please try to open it in incognito mode.

### 5. Wait until you get notified about your assigned device IP [MacOS]

After someone picked up your public key and added it to our WireGuard servers you will get your assigned device IP back via your **rlim.com** document.

### 6. Configure Gnosis VPN service configuration - hoprd node [MacOS]

1. Download [config](./config.toml) and place it next to the downloaded binary file.
2. Open `config.toml` in edit mode and locate `[hoprd_node]` section to adjust these values:

```toml
[hoprd_node]
endpoint = "http://123.456.7.89:3002"
api_token = "<hoprd node API token>"

internal_connection_port = 60006
```

`endpoint` is the URL (including port) pointing to the API access of your node (e.g., `http://123.456.7.89:3002`).
`api_token` is the API access token of your node.
`internal_connection_port` is the static UDP port of your hoprd node on which Gnosis VPN will establish a connection.

Note: If you have a firewall running on your hoprd node, you need to update your port forwarding rules accordingly.

If you like a more extensively documented configuration file try using [documented config](./documented-config.toml).

### 7. Configure Gnosis VPN service configuration - exit location [MacOS]

Visit [GnosisVPN servers](https://gnosisvpn.com/servers) and choose an exit location.
Copy the exit node peer id into your `config.toml` or update parameters manually:

```toml
# copy this section from https://gnosisvpn.com/servers
[connection]
destination = "<exit node peer id>"
```

Save and close the configuration file.

### 8. Launch the GnosisVPN binary file [MacOS]

1. Return to your downloaded binary file and make it executable:

```bash
chmod +x ./gnosis-vpn-aarch64-darwin
# depending on your system, alternatively: chmod +x ./gnosis-vpn-x86_64-darwin
```

2. Provide the path to your configuration file and a socket path to start the GnosisVPN binary file.
The socket path is only used for communication with the GnosisVPN service which is out of scope for this guide.
   If you do not want to provide a socket path, you can also start the binary file with privileged access and it will use `/var/run/gnosisvpn.sock` as it's communication socket.

```bash
# <system> matches the one you chose earlier
GNOSISVPN_CONFIG_PATH=./config.toml GNOSISVPN_SOCKET_PATH=./gnosisvpn.sock ./gnosis-vpn-aarch64-darwin
# depending on your system, alternatively: GNOSISVPN_CONFIG_PATH=./config.toml GNOSISVPN_SOCKET_PATH=./gnosisvpn.sock ./gnosis-vpn-x86_64-darwin

# or with privileged access
sudo GNOSISVPN_CONFIG_PATH=./config.toml ./gnosis-vpn-aarch64-darwin`
# depending on your system, alternatively: sudo GNOSISVPN_CONFIG_PATH=./config.toml ./gnosis-vpn-x86_64-darwin`
```

3. Because of macOS security settings, you will see a message that says binary file “cannot be opened because the developer cannot be verified”.
   Click "Cancel" or "Done", then open System Settings → Privacy & Security, scroll down to Security, and find the blocked binary file. Click "Allow Anyway".

4. In your terminal, run the command to start the binary file again. MacOS will prompt you one more time to confirm if you want to open it. Click "Open" or "Open anyway".

If you see immediate errors on startup it is most likely due to errors in your configuration settings.
The binary file should tell you which setting parameter might be wrong.

### 10. Edit the newly created WireGuard tunnel [MacOS]

In the WireGuard app, edit the tunnel you created:

```conf
[Interface]
PrivateKey = <Generated automatic by WireGuard app>
Address = <device IP - received via drop location, e.g.: 20.0.0.5/32>

[Peer]
PublicKey = <wg server pub key - listed on https://gnosisvpn.com/servers>
Endpoint = <hoprd node IP:60006 - the port needs to match your `internal_connection_port` configuraiton>
AllowedIPs = <what traffic do you want to route - usually the base of device IP would be a good start, e.g.: 20.0.0.0/24, set to 0.0.0.0/0 to route all traffic>
PersistentKeepalive = 30
```

---

## Instructions for Linux

### 1. Download the latest binary file [Linux]

Download the latest GnosisVPN binary file for your system by visiting the [GitHub releases](https://github.com/hoprnet/gnosis-vpn-client-system-service/releases) page.
Choose the binary file that matches your system:

| system                    | binary file                     |
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

1. Go to rlim.com.
2. Locate the "Custom URL" input field and enter your desired text (e.g., `toms-feedback-gvpn`).
3. Copy the updated URL from the browser's address bar (e.g., `https://rlim.com/toms-feedback-gvpn`).
4. Copy the edit code displayed on the top line of the page.

### 4. Provide necessary data to be eligible for GnosisVPN PoC demo [Linux]

1. Preview public key:

```bash
cat publickey | xclip -r -sel clip
```

2. Provide your public key, the **rlim.com** URL, and the edit code in our [onboarding form](https://cryptpad.fr/form/#/2/form/view/bigkDtjj+9G3S4DWCHPTOjfL70MJXdEWTDjkZRrUH9Y/).
   If you have trouble opening cryptpad, please try to open it in incognito mode.

### 5. Wait until you get notified about your assigned device IP [Linux]

After someone picked up your public key and added it to our WireGuard servers you will get your assigned device IP back via your **rlim.com** document.

### 6. Configure Gnosis VPN service configuration - hoprd node [Linux]

1. Copy [documented config](./documented-config.toml) to `/etc/gnosisvpn/config.toml` and open it in edit mode.
   If you don't like the configuration file location you can override this default via `GNOSISVPN_CONFIG_PATH` env var.

2. Uncomment `[hoprd_node]` section and adjust values as needed:

```toml
## hoprd node section - your hoprd node that acts as the connection entry point
# [hoprd_node]

# # URL pointing to API access of your node with schema and port (e.g.: `http://123.456.7.89:3002`)
# endpoint = "<hoprd node API endpoint>"

# # API access token
# api_token = "<hoprd node API token>"
```

### 7. Configure Gnosis VPN service configuration - exit location [Linux]

Visit [GnosisVPN servers](https://gnosisvpn.com/servers) and choose an exit location.
Update parameters in `/etc/gnosisvpn/config.toml`:

```toml
# # copy this section from https://gnosisvpn.com/servers
# [connection]

# # the exit peer id (where the connection should terminate)
# destination = "<exit node peer id>"
```

### 8. Configure Gnosis VPN service configuration - static port configuration [Linux]

You can configure a GnosisVPN connection to run on a static port on your hoprd node.
This is useful if you set up a firewall rule to allow traffic on specific ports only.
Go back to the `[hoprd_node]` section and have a look at the optional `internal_connection_port` parameter.
Uncomment it like shown in this example to provide your static port.

```toml
[hoprd_node]

# ... (endpoint and api_token configs)

# [OPTIONAL] internal port - use this if you have a firewall running and only forward a specific port
# this is NOT your API port which must be specified in the `endpoint` field
# this port is an addiontal port used to establish the tunnel connection on your hoprd node
# in general if you want to establish a connection on specific port, just provide this port here
internal_connection_port = 60006
```

### 9. Ready to start the GnosisVPN binary file [Linux]

Replace `<gnosis-vpn-binary>` with the binary file you downloaded earlier, see [step 1](#1-download-the-latest-binary-linux).

1. Return to your downloaded binary file and make it executable:

```bash
chmod +x <gnosis-vpn-binary>
```

2. Launch GnosisVPN binary file

```bash
# with privileged access
sudo <gnosis-vpn-binary>
# without privileged access
GNOSISVPN_CONFIG_PATH=./config.toml GNOSISVPN_SOCKET_PATH=./gnosisvpn.sock <gnosis-vpn-binary>
```

If you see immediate errors on startup it is most likely due to errors in your configuration settings.
The binary file should tell you which setting parameter might be wrong.

### 10. Create a wireguard interface and use the established GnosisVPN connection [Linux]

Create a file called `wg-gnosisvpn-beta.conf` inside `/etc/wireguard/` with the following content:

```conf
[Interface]
PrivateKey = <content privatekey>
Address = <device IP - received via drop location, e.g.: 20.0.0.5/32>

[Peer]
PublicKey = <wg server pub key - listed on https://gnosisvpn.com/servers>
Endpoint = <hoprd node IP:60006 - the port needs to match your `internal_connection_port` configuraiton>
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
# Caution: this section is experimental at best and will only work on Linux
# this section holds the wireguard specific settings
[wireguard]
# local interface IP, onboarding info will provide this
address = "10.34.0.8/32"
# wireguard server public peer id - onboarding info will provide this
server_public_key = "<wg server public peer id>"
```

At this point the you might see some notificaiton that a `wg0-gnosisvpn` interface is now connected.
The GnosisVPN connection was opened by the service and will kept open.
Wireguard is also connected and you will be able to use a socks5 proxy on your device.
