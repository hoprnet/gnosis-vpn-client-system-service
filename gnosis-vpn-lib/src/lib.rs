use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use url::Url;

#[derive(Serialize, Deserialize, Clone)]
pub enum Command {
    Status,
    EntryNode { endpoint: Url, api_token: String },
    ExitNode { peer_id: String },
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c = match self {
            Command::EntryNode {
                endpoint,
                api_token: _,
            } => Command::EntryNode {
                endpoint: endpoint.clone(),
                api_token: "*****".to_string(),
            },
            c => c.clone(),
        };
        let s = serde_json::to_string(&c).unwrap();
        write!(f, "{}", s)
    }
}

impl FromStr for Command {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
