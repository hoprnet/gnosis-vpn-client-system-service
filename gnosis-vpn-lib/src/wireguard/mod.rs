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
}

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
}
