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

const TMP_FILE: &str = "wg0_gnosisvpn.conf";

impl WireGuard for Tooling {
    fn generate_key(&self) -> Result<String, Error> {
        let output = Command::new("wg")
            .arg("genkey")
            .output()
            .map_err(|e| Error::IO(e.to_string()))?;
        String::from_utf8(output.stdout)
            .map(|s| s.trim().to_string())
            .map_err(|e| Error::FromUtf8Error(e))
    }

    fn connect_session(&self, session: &SessionInfo) -> Result<(), Error> {
        let p_dirs = dirs::project().ok_or(Error::IO("unable to create project directories".to_string()))?;
        let cache_dir = p_dirs.cache_dir();
        fs::create_dir_all(cache_dir).map_err(|e| Error::IO(e.to_string()))?;

        let conf_file = cache_dir.join(TMP_FILE);
        let config = session.to_file_string();
        tracing::info!(file = ?conf_file, "ser: {:?}", config);
        let content = config.as_bytes();
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

impl SessionInfo {
    fn to_file_string(&self) -> String {
        let allowed_ips = self
            .interface
            .address
            .split('.')
            .take(3)
            .collect::<Vec<&str>>()
            .join(".")
            + ".0/24";
        format!(
            "[Interface]
PrivateKey = {private_key}
Address = {address}

[Peer]
PublicKey = {public_key}
Endpoint = {endpoint}
AllowedIPs = {allowed_ips}
PersistentKeepalive = 30
",
            private_key = self.interface.private_key,
            address = self.interface.address,
            public_key = self.peer.public_key,
            endpoint = self.peer.endpoint,
            allowed_ips = allowed_ips
        )
    }
}
