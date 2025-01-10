use crate::wireguard::{Error, SessionInfo, WireGuard};

#[derive(Debug)]
pub struct UserSpace {}

pub fn available() -> Result<bool, Error> {
    Err(Error::NotYetImplemented("userspace".to_string()))
}

impl UserSpace {
    pub fn new() -> Self {
        UserSpace {}
    }
}

impl WireGuard for UserSpace {
    fn generate_key(&self) -> Result<String, Error> {
        Err(Error::NotYetImplemented("userspace".to_string()))
    }

    fn connect_session(&self, _session: &SessionInfo) -> Result<(), Error> {
        Err(Error::NotYetImplemented("connect_session".to_string()))
    }
}
