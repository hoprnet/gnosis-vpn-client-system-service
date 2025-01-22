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
    pub interface: InterfaceInfo,
    pub peer: PeerInfo,
}

struct InterfaceInfo {
    pub private_key: String,
    pub address: String,
    pub allowed_ips: Option<String>,
}

struct PeerInfo {
    pub public_key: String,
    pub endpoint: String,
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
