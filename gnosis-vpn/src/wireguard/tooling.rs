use std::process::Command;

use crate::wireguard::Wireguard;

#[derive(Debug)]
pub struct Tooling {}

impl Tooling {
    pub fn new() -> Self {
        Tooling {}
    }
}

impl Wireguard for Tooling {
    fn available(&self) -> bool {
        let res = Command::new("which").arg("wg-quick").status();
        match res {
            Ok(code) => code.success(),
            Err(e) => {
                tracing::warn!(warn = ?e, "failed checking for wg-quick");
                false
            }
        }
    }
}
