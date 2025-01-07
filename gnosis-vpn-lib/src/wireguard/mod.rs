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
}

pub fn best_flavor() -> (Option<Box<dyn WireGuard>>, Vec<Error>) {
    let mut errors: Vec<Error> = Vec::new();

    match kernel::available() {
        Ok(true) => return (Some(Box::new(kernel::Kernel::new())), errors),
        Ok(false) => (),
        Err(e) => errors.push(Error::IO(e)),
    }

    match userspace::available() {
        Ok(true) => return (Some(Box::new(userspace::UserSpace::new())), errors),
        Ok(false) => (),
        Err(e) => errors.push(Error::IO(e)),
    }

    match tooling::available() {
        Ok(true) => return (Some(Box::new(tooling::Tooling::new())), errors),
        Ok(false) => (),
        Err(e) => errors.push(Error::IO(e)),
    }

    (None, errors)
}

pub trait WireGuard {}
