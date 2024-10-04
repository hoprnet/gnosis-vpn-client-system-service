use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fmt;

#[derive(Serialize, Deserialize)]
pub struct WgConnect {
    pub peer: String,
    pub allowed_ips: String,
    pub endpoint: String,
}

impl fmt::Display for WgConnect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Wireguard Connect [peer: {}, allowed_ips: {}, endpoint: {}]",
            self.peer, self.allowed_ips, self.endpoint
        )
    }
}
impl WgConnect {
    pub fn serialize(&self) -> Result<String> {
        serde_json::to_string(&self)
    }
}
