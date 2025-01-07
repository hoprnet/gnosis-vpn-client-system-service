mod kernel;
mod tooling;
mod userspace;

pub fn best_flavor() -> Option<Box<dyn Wireguard>> {
    if kernel::available() {
        Some(Box::new(kernel::Kernel::new()))
    } else if userspace::available() {
        Some(Box::new(userspace::UserSpace::new()))
    } else if tooling::available() {
        Some(Box::new(tooling::Tooling::new()))
    } else {
        None
    }
}

pub trait Wireguard {
    fn available(&self) -> bool;
}
