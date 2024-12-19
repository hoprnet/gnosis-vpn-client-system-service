use serde_derive::{Deserialize, Serialize};
use std::vec::Vec;
use std::default::Default;
use url::{Host, Url};

#[derive(Default, Serialize, Deserialize)]
struct Config {
    version: u8,
    entrynode: EntryNodeConfig,
    session: SessionConfig,
    wireguard: WireguardConfig,
}

#[derive(Serialize, Deserialize)]
struct SessionConfig {
    target: SessionTargetConfig,
    capabilites: Vec<CapabilitiesConfig>,
}


#[derive(Serialize, Deserialize)]
struct EntryNodeConfig {
    endpoint: Url,
    api_token: String,
}

#[derive(Serialize, Deserialize)]
struct SessionTargetConfig {
    type: SessionTargetType,
    endpoint: Host,
}

enum CapabilitiesConfig {
    Segmentation,
    Retransmission,
}


#[cfg(target_family = "linux")]
pub fn config_path() -> PathBuf {
    PathBuf::from("/etc/gnosisvpn/config.yaml")
}
