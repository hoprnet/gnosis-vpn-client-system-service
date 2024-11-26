use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use url::Url;

mod socket;
pub use socket::socket_path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Command {
    Status,
    EntryNode {
        endpoint: Url,
        api_token: String,
        session_port: Option<u16>,
    },
    ExitNode {
        peer_id: String,
    },
}

impl Command {
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c = match self {
            Command::EntryNode {
                session_port, endpoint, ..
            } => Command::EntryNode {
                endpoint: endpoint.clone(),
                api_token: "*****".to_string(),
                session_port: session_port.clone(),
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
