use serde::Serialize;
use std::fs;
use std::process::{Command, Stdio};

use crate::dirs;
use crate::wireguard::{Error, SessionInfo, WireGuard};

#[derive(Debug)]
pub struct Tooling {}

pub fn available() -> Result<bool, Error> {
    let code = Command::new("which")
        .arg("wg-quick")
        // suppress log output
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| Error::IO(e.to_string()))?;
    Ok(code.success())
}

impl Tooling {
    pub fn new() -> Self {
        Tooling {}
    }
}

const TMP_FILE: &str = "wg0-quick-gnosisvpn.conf";

#[derive(Serialize)]
pub struct Config {
    Interface: InterfaceConfig,
    Peer: PeerConfig,
}

#[derive(Serialize)]
struct InterfaceConfig {
    PrivateKey: String,
    Address: String,
}

#[derive(Serialize)]
struct PeerConfig {
    PublicKey: String,
    Endpoint: String,
    AllowedIPs: String,
    PersistentKeepalive: u16,
}

impl WireGuard for Tooling {
    fn generate_key(&self) -> Result<String, Error> {
        let output = Command::new("wg")
            .arg("genkey")
            .output()
            .map_err(|e| Error::IO(e.to_string()))?;
        String::from_utf8(output.stdout).map_err(|e| Error::FromUtf8Error(e))
    }

    fn connect_session(&self, session: &SessionInfo) -> Result<(), Error> {
        let p_dirs = dirs::project().ok_or(Error::IO("unable to create project directories".to_string()))?;
        let cache_dir = p_dirs.cache_dir();
        fs::create_dir_all(cache_dir).map_err(|e| Error::IO(e.to_string()))?;

        let conf_file = cache_dir.join(TMP_FILE);
        let config = Config::from(session);
        let ser = toml::to_string(&config).map_err(|e| Error::Toml(e))?;
        tracing::info!("ser: {:?}", ser);
        let content = ser.as_bytes();
        fs::write(&conf_file, content).map_err(|e| Error::IO(e.to_string()))?;

        let output = Command::new("wg-quick")
            .arg("up")
            .arg(conf_file)
            .output()
            .map_err(|e| Error::IO(e.to_string()))?;

        tracing::info!("wg-quick up output: {:?}", output);
        Ok(())
    }
}

impl From<&SessionInfo> for Config {
    fn from(session: &SessionInfo) -> Self {
        let allowed_ips = session
            .interface
            .address
            .split('.')
            .take(3)
            .collect::<Vec<&str>>()
            .join(".")
            + ".0/24";
        Config {
            Interface: InterfaceConfig {
                PrivateKey: session.interface.private_key.clone(),
                Address: session.interface.address.clone(),
            },
            Peer: PeerConfig {
                PublicKey: session.peer.public_key.clone(),
                Endpoint: session.peer.endpoint.clone(),
                AllowedIPs: allowed_ips,
                PersistentKeepalive: 30,
            },
        }
    }
}
