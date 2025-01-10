use std::fmt::Debug;
use thiserror::Error;

mod kernel;
mod tooling;
mod userspace;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("implementation pending: {0}")]
    NotYetImplemented(String),
    // cannot use IO error because it does not allow Clone or Copy
    #[error("IO error: {0}")]
    IO(String),
    #[error("encoding error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("toml error: {0}")]
    Toml(#[from] toml::ser::Error),
}

pub struct SessionInfo {
    interface: InterfaceInfo,
    peer: PeerInfo,
}

struct InterfaceInfo {
    private_key: String,
    address: String,
}

struct PeerInfo {
    public_key: String,
    endpoint: String,
}

pub fn best_flavor() -> (Option<Box<dyn WireGuard>>, Vec<Error>) {
    let mut errors: Vec<Error> = Vec::new();

    match kernel::available() {
        Ok(true) => return (Some(Box::new(kernel::Kernel::new())), errors),
        Ok(false) => (),
        Err(e) => errors.push(e),
    }

    match userspace::available() {
        Ok(true) => return (Some(Box::new(userspace::UserSpace::new())), errors),
        Ok(false) => (),
        Err(e) => errors.push(e),
    }

    match tooling::available() {
        Ok(true) => return (Some(Box::new(tooling::Tooling::new())), errors),
        Ok(false) => (),
        Err(e) => errors.push(e),
    }

    (None, errors)
}

pub trait WireGuard: Debug {
    fn generate_key(&self) -> Result<String, Error>;
    fn connect_session(&self, session: SessionInfo) -> Result<(), Error>;
}

impl SessionInfo {
    pub fn new(if_private_key: &str, if_address: &str, peer_public_key: &str, peer_endpoint: &str) -> Self {
        Self {
            interface: InterfaceInfo {
                private_key: if_private_key.to_string(),
                address: if_address.to_string(),
            },
            peer: PeerInfo {
                public_key: peer_public_key.to_string(),
                endpoint: peer_endpoint.to_string(),
            },
        }
    }
}
