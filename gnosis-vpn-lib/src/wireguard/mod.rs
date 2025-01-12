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
    #[error("monitoring error: {0}")]
    Monitoring(String),
    #[error("wireguard error: {0}")]
    WgError(String),
}

pub struct ConnectSession {
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

/*
pub struct VerifySession {
    peer_public_key: String,
    private_key: String,
}
*/

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
    fn connect_session(&self, session: &ConnectSession) -> Result<(), Error>;
    fn public_key(&self, priv_key: &str) -> Result<String, Error>;
    // fn close_session(&self) -> Result<(), Error>;
    // fn verify_session(&self, session: &VerifySession) -> Result<(), Error>;
}

impl ConnectSession {
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

/*
impl VerifySession {
    pub fn new(peer_public_key: &str, private_key: &str) -> Self {
        Self {
            peer_public_key: peer_public_key.to_string(),
            private_key: private_key.to_string(),
        }
    }
}
*/
