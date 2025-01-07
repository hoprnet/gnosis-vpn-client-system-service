use std::io::Error;
use std::io::ErrorKind;

use crate::wireguard::WireGuard;

// This will be the implementation using netlink kernel access.
#[derive(Debug)]
pub struct Kernel {}

pub fn available() -> Result<bool, Error> {
    Err(Error::new(
        ErrorKind::Other,
        "netlink kernel module not yet implemented",
    ))
}

impl Kernel {
    pub fn new() -> Self {
        Kernel {}
    }
}

impl WireGuard for Kernel {}
