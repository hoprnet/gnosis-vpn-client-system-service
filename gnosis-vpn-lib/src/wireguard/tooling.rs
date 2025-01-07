use std::io::Error;
use std::process::Command;

use crate::wireguard::WireGuard;

#[derive(Debug)]
pub struct Tooling {}

pub fn available() -> Result<bool, Error> {
    let code = Command::new("which").arg("wg-quick").status()?;
    Ok(code.success())
}

impl Tooling {
    pub fn new() -> Self {
        Tooling {}
    }
}

impl WireGuard for Tooling {}
