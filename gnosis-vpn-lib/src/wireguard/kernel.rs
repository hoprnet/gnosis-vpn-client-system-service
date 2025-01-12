use crate::wireguard::{ConnectSession, Error, WireGuard};

// This will be the implementation using netlink kernel access.
#[derive(Debug)]
pub struct Kernel {}

pub fn available() -> Result<bool, Error> {
    Err(Error::NotYetImplemented("netlink kernel module".to_string()))
}

impl Kernel {
    pub fn new() -> Self {
        Kernel {}
    }
}

impl WireGuard for Kernel {
    fn generate_key(&self) -> Result<String, Error> {
        Err(Error::NotYetImplemented("netlink kernel module".to_string()))
    }

    fn connect_session(&self, _session: &ConnectSession) -> Result<(), Error> {
        Err(Error::NotYetImplemented("connect_session".to_string()))
    }

    fn public_key(&self, _priv_key: &str) -> Result<String, Error> {
        Err(Error::NotYetImplemented("connect_session".to_string()))
    }
}
