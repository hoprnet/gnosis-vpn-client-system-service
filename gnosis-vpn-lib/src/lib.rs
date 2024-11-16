use anyhow::Context;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize)]
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
