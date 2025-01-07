use std::io::Error;
use std::io::ErrorKind;

use crate::wireguard::WireGuard;

#[derive(Debug)]
pub struct UserSpace {}

pub fn available() -> Result<bool, Error> {
    Err(Error::new(
        ErrorKind::Other,
        "userspace wireguard handling not yet implemented",
    ))
}

impl UserSpace {
    pub fn new() -> Self {
        UserSpace {}
    }
}

impl WireGuard for UserSpace {}
