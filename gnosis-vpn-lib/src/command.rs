use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::fmt;
use std::str::FromStr;
use url::Url;

use crate::log_output;

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Command {
    Status,
    EntryNode {
        endpoint: Url,
        api_token: String,
        listen_host: Option<String>,
        hop: Option<u8>,
        #[serde_as(as = "Option<DisplayFromStr>")]
        intermediate_id: Option<PeerId>,
    },
    ExitNode {
        #[serde_as(as = "DisplayFromStr")]
        peer_id: PeerId,
    },
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c = match self {
            Command::EntryNode {
                hop,
                listen_host,
                endpoint,
                api_token: _,
                intermediate_id,
            } => Command::EntryNode {
                endpoint: endpoint.clone(),
                api_token: "*****".to_string(),
                listen_host: listen_host.clone(),
                hop: *hop,
                intermediate_id: *intermediate_id,
            },
            c => c.clone(),
        };
        let s = log_output::serialize(&c);
        write!(f, "{}", s)
    }
}

impl FromStr for Command {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
