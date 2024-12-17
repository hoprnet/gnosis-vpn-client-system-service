use anyhow::anyhow;
use std::process::Command;

use crate::task::Task;

#[derive(Debug)]
pub struct Tooling {
    available: bool,
    error: Option<anyhow::Error>,
}

impl Tooling {
    pub fn new() -> Self {
        Tooling {
            available: false,
            error: None,
        }
    }
}

impl Task for Tooling {
    fn init(&mut self) {
        let res = Command::new("which").arg("wg-quick").status();
        match res {
            Ok(code) => self.available = code.success(),
            Err(e) => self.error = Some(anyhow!(e)),
        }
    }

    fn run(&self) {}
}
