use crate::wireguard::Wireguard;

// This will be the implementation using netlink kernel access.
#[derive(Debug)]
pub struct Kernel {}

impl Kernel {
    pub fn new() -> Self {
        Kernel {}
    }
}

impl Wireguard for Kernel {
    fn available(&self) -> bool {
        // TODO
        false
    }
}
