use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct State {
    pub wg_private_key: Option<String>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("state folder error")]
    NoStateFolder,
    #[error("state file not found")]
    NoFile,
    #[error("error determining parent folder")]
    NoParentFolder,
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("error serializing/deserializing state: {0}")]
    BinCodeError(#[from] bincode::Error),
}

fn path() -> Option<PathBuf> {
    let project_dirs = ProjectDirs::from("org", "hoprnet", "gnosisvpn")?;
    let data_dir = project_dirs.data_local_dir();
    let state_file = data_dir.join("state.bin");
    Some(state_file)
}

pub fn read() -> Result<State, Error> {
    let p = match path() {
        Some(p) => p,
        None => return Err(Error::NoStateFolder),
    };
    let content = fs::read(p).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::NoFile
        } else {
            Error::IO(e)
        }
    })?;
    let state: State = bincode::deserialize(&content[..]).map_err(|e| Error::BinCodeError(e))?;
    Ok(state)
}

impl State {
    pub fn set_wg_private_key(&mut self, key: String) -> Result<(), Error> {
        self.wg_private_key = Some(key);
        self.write()
    }

    fn write(&self) -> Result<(), Error> {
        let path = match path() {
            Some(p) => p,
            None => return Err(Error::NoStateFolder),
        };
        let content = bincode::serialize(&self).map_err(|e| Error::BinCodeError(e))?;
        let parent = path.parent().ok_or(Error::NoParentFolder)?;
        fs::create_dir_all(parent).map_err(|e| Error::IO(e))?;
        fs::write(path, content).map_err(|e| Error::IO(e))
    }
}

impl Default for State {
    fn default() -> Self {
        State { wg_private_key: None }
    }
}
