use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::default::Default;
use std::fs;
use std::path::PathBuf;
use std::vec::Vec;
use thiserror::Error;
use url::Host;

use crate::peer_id::PeerId;

const SUPPORTED_CONFIG_VERSIONS: [u8; 1] = [1];

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    version: u8,
    entrynode: Option<EntryNodeConfig>,
    session: Option<SessionConfig>,
    wireguard: Option<WireguardConfig>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct EntryNodeConfig {
    endpoint: (Host, u16),
    api_token: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionConfig {
    capabilites: Vec<CapabilitiesConfig>,
    destination: PeerId,
    listen_host: Option<String>,
    path: Option<SessionPathConfig>,
    target: SessionTargetConfig,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct WireguardConfig {
    private_key: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionTargetConfig {
    type_: SessionTargetType,
    endpoint: (Host, u16),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum CapabilitiesConfig {
    Segmentation,
    Retransmission,
}

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
pub enum SessionTargetType {
    #[default]
    Plain,
    Sealed,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
            entrynode: None,
            session: None,
            wireguard: None,
        }
    }
}
