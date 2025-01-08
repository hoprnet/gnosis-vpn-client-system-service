use std::process::{Command, Stdio};

use crate::wireguard::{Error, WireGuard};

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

impl WireGuard for Tooling {
    fn generate_key(&self) -> Result<String, Error> {
        let output = Command::new("wg")
            .arg("genkey")
            .output()
            .map_err(|e| Error::IO(e.to_string()))?;
        String::from_utf8(output.stdout).map_err(|e| Error::FromUtf8Error(e))
    }
}
