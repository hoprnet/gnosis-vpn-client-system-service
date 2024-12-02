use anyhow::Context;
use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::fmt;
use std::str::FromStr;
use url::Url;

mod socket;
pub use socket::socket_path;

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Command {
    Status,
    EntryNode {
        endpoint: Url,
        api_token: String,
        listen_host: Option<String>,
        hop: u8,
    },
    ExitNode {
        #[serde_as(as = "DisplayFromStr")]
        peer_id: PeerId,
    },
}

impl Command {
    pub fn to_json_string(&self) -> anyhow::Result<String> {
        serde_json::to_string(self).context("Failed to serialize command")
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c = match self {
            Command::EntryNode {
                hop,
                listen_host,
                endpoint,
                ..
            } => Command::EntryNode {
                endpoint: endpoint.clone(),
                api_token: "*****".to_string(),
                listen_host: listen_host.clone(),
                hop: hop.clone(),
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
