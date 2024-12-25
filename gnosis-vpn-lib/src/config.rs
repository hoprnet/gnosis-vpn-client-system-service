use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::default::Default;
use std::fs;
use std::path::PathBuf;
use std::vec::Vec;
use thiserror::Error;
use url::Host;

const SUPPORTED_CONFIG_VERSIONS: [u8; 1] = [1];

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    version: u8,
    entrynode: Option<EntryNodeConfig>,
    session: SessionConfig,
    wireguard: Option<WireguardConfig>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct EntryNodeConfig {
    endpoint: (Host, u16),
    api_token: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionConfig {
    target: SessionTargetConfig,
    capabilites: Vec<CapabilitiesConfig>,
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
            session: SessionConfig::default(),
            wireguard: None,
        }
    }
}

impl Default for SessionTargetConfig {
    fn default() -> Self {
        SessionTargetConfig {
            type_: SessionTargetType::default(),
            endpoint: (Host::Domain("wireguard.staging.hoprnet.link".to_string()), 51820),
        }
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        SessionConfig {
            target: SessionTargetConfig::default(),
            capabilites: vec![CapabilitiesConfig::Segmentation],
        }
    }
}
