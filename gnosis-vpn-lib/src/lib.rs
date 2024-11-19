use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fmt;
use url::Url;

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub enum Command {
    Status,
    EntryNode { endpoint: Url, api_token: String },
}

pub fn to_cmd(data: &str) -> anyhow::Result<Command> {
    serde_json::from_str(data).with_context(|| format!("unable to parse command: {}", data))
}

pub fn to_string(cmd: &Command) -> anyhow::Result<String> {
    serde_json::to_string(cmd).with_context(|| "unable to serialize command")
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c =
        match self {
            Command::EntryNode { endpoint, api_token: _  } => Command::EntryNode { endpoint: endpoint.clone(), api_token: "*****".to_string() },
            c => c.clone(),
        };
        let s = serde_json::to_string(&c).unwrap();
        write!(f, "{}", s)
    }
}
