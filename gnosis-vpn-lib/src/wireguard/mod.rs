use thiserror::Error;

mod kernel;
mod tooling;
mod userspace;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Implementation pending")]
    NotYetImplemented,
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Deserialization error: {0}")]
    Deserialization(#[from] toml::de::Error),
    #[error("Unsupported config version")]
    VersionMismatch(u8),
}

pub fn best_flavor() -> Result<Option<Box<dyn WireGuard>>, Error> {
    if kernel::available()? {
        Ok(Some(Box::new(kernel::Kernel::new())))
    } else if userspace::available()? {
        Ok(Some(Box::new(userspace::UserSpace::new())))
    } else if tooling::available()? {
        Ok(Some(Box::new(tooling::Tooling::new())))
    } else {
        Ok(None)
    }
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

pub trait WireGuard {}
