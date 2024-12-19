use serde::{Deserialize, Serialize};
use std::default::Default;
use std::path::PathBuf;
use std::vec::Vec;
use url::Host;

#[derive(Serialize, Deserialize)]
struct Config {
    version: u8,
    entrynode: Option<EntryNodeConfig>,
    session: SessionConfig,
    wireguard: Option<WireguardConfig>,
}

#[derive(Serialize, Deserialize)]
struct EntryNodeConfig {
    endpoint: (Host, u16),
    api_token: String,
}

#[derive(Serialize, Deserialize)]
struct SessionConfig {
    target: SessionTargetConfig,
    capabilites: Vec<CapabilitiesConfig>,
}

#[derive(Serialize, Deserialize)]
struct WireguardConfig {
    private_key: String,
}

#[derive(Serialize, Deserialize)]
struct SessionTargetConfig {
    type_: SessionTargetType,
    endpoint: (Host, u16),
}

#[derive(Serialize, Deserialize)]
enum CapabilitiesConfig {
    Segmentation,
    Retransmission,
}

#[derive(Default, Serialize, Deserialize)]
enum SessionTargetType {
    #[default]
    Plain,
    Sealed,
}

#[cfg(target_os = "linux")]
pub fn path() -> PathBuf {
    PathBuf::from("/etc/gnosisvpn/config.yaml")
}

impl Default for Config {
    fn default() -> Self {
        Config {
            version: 1,
            entrynode: None,
            session: SessionConfig {
                target: SessionTargetConfig {
                    type_: SessionTargetType::Plain,
                    endpoint: (Host::Domain("wireguard.staging.hoprnet.link".to_string()), 51820),
                },
                capabilites: vec![CapabilitiesConfig::Segmentation],
            },
            wireguard: None,
        }
    }
}
