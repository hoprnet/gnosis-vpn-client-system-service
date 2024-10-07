use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
pub enum Command {
    // TODO response message
    WgConnect {
        peer: String,
        allowed_ips: String,
        endpoint: String,
    },
}

pub fn to_cmd(data: &str) -> Result<Command> {
    serde_json::from_str(data)
}

pub fn to_string(cmd: &Command) -> Result<String> {
    serde_json::to_string(cmd)
}
