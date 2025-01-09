use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::default::Default;
use std::fs;
use std::path::PathBuf;
use std::vec::Vec;
use thiserror::Error;
use url::{Host, Url};

use crate::peer_id::PeerId;

const SUPPORTED_CONFIG_VERSIONS: [u8; 1] = [1];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub version: u8,
    pub entry_node: Option<EntryNodeConfig>,
    pub session: Option<SessionConfig>,
    pub wire_guard: Option<WireGuardConfig>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EntryNodeConfig {
    pub endpoint: Url,
    pub api_token: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionConfig {
    pub capabilites: Vec<CapabilitiesConfig>,
    pub destination: PeerId,
    pub listen_host: Option<String>,
    pub path: Option<SessionPathConfig>,
    pub target: SessionTargetConfig,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WireGuardConfig {
    pub private_key: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionTargetConfig {
    pub type_: SessionTargetType,
    pub host: Host,
    pub port: u16,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CapabilitiesConfig {
    Segmentation,
    Retransmission,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub enum SessionTargetType {
    #[default]
    Plain,
    Sealed,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SessionPathConfig {
    Hop(u8),
    IntermediateId(PeerId),
}

#[cfg(target_os = "linux")]
pub fn path() -> PathBuf {
    PathBuf::from("/etc/gnosisvpn/config.toml")
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Config file not found")]
    NoFile,
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Deserialization error: {0}")]
    Deserialization(#[from] toml::de::Error),
    #[error("Unsupported config version")]
    VersionMismatch(u8),
}

pub fn read() -> Result<Config, Error> {
    let content = fs::read_to_string(path()).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::NoFile
        } else {
            Error::IO(e)
        }
    })?;
    let config: Config = toml::from_str(&content).map_err(|e| Error::Deserialization(e))?;
    if SUPPORTED_CONFIG_VERSIONS.contains(&config.version) {
        Ok(config)
    } else {
        Err(Error::VersionMismatch(config.version))
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            version: 1,
            entry_node: None,
            session: None,
            wire_guard: None,
        }
    }
}

impl Default for SessionPathConfig {
    fn default() -> Self {
        SessionPathConfig::Hop(1)
    }
}
