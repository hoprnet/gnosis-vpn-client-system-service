use crate::wireguard::Wireguard;

#[derive(Debug)]
pub struct UserSpace {}

impl UserSpace {
    pub fn new() -> Self {
        UserSpace {}
    }
}

impl Wireguard for UserSpace {
    fn available(&self) -> bool {
        // TODO
        false
    }
}
