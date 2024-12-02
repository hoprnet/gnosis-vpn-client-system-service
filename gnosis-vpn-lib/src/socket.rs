use crate::command::Command;
use crate::error::Error;
use std::io::{Read, Write};
use std::os::unix::net;
use std::path::PathBuf;

pub enum ReturnValue {
    WithResponse(String),
    NoResponse,
}

pub fn process_cmd(cmd: &Command) -> Result<ReturnValue, Error> {
    let sock_path = socket_path();

    tracing::debug!(socket_path = ?sock_path, "using socket path");
    check_path(&sock_path)?;
    tracing::debug!(sock_path = ?sock_path, "socket path verified");

    let mut stream = connect_stream(&sock_path)?;
    tracing::debug!(?stream, "stream connected");

    let json_cmd = serialize_command(cmd)?;
    tracing::debug!(?json_cmd, "command serialized");

    push_command(&mut stream, &json_cmd)?;
    tracing::debug!(?json_cmd, "command pushed");

    if let Command::Status = cmd {
        let response = pull_response(&mut stream)?;
        tracing::debug!(?response, "response pulled");
        Ok(ReturnValue::WithResponse(response))
    } else {
        Ok(ReturnValue::NoResponse)
    }
}

fn check_path(sock_path: &PathBuf) -> Result<(), Error> {
    match sock_path.try_exists() {
        Ok(true) => Ok(()),
        Ok(false) => Err(Error::ServiceNotRunning),
        Err(x) => Err(Error::SocketPathIO(x)),
    }
}

fn connect_stream(sock_path: &PathBuf) -> Result<net::UnixStream, Error> {
    match net::UnixStream::connect(sock_path) {
        Ok(socket) => Ok(socket),
        Err(x) => Err(Error::ConnectSocketIO(x)),
    }
}

fn serialize_command(cmd: &Command) -> Result<String, Error> {
    match serde_json::to_string(&cmd) {
        Ok(cmd_as_json) => Ok(cmd_as_json),
        Err(x) => Err(Error::CommandSerialization(x)),
    }
}

fn push_command(socket: &mut net::UnixStream, json_cmd: &str) -> Result<(), Error> {
    // flush is not enough to push the command
    // we need to shutdown the write channel to signal the other side that all data was transferred
    let res = socket
        .write_all(json_cmd.as_bytes())
        .map(|_| socket.flush())
        .map(|_| socket.shutdown(std::net::Shutdown::Write));

    match res {
        Ok(_) => Ok(()),
        Err(x) => Err(Error::WriteSocketIO(x)),
    }
}

fn pull_response(socket: &mut net::UnixStream) -> Result<String, Error> {
    let mut response = String::new();
    let res = socket.read_to_string(&mut response);
    match res {
        Ok(_) => Ok(response),
        Err(x) => Err(Error::ReadSocketIO(x)),
    }
}

#[cfg(target_family = "unix")]
fn socket_path() -> PathBuf {
    PathBuf::from("/var/run/gnosis-vpn.sock")
}

// #[cfg(target_family = "windows")]
// fn socket_path() -> PathBuf {
// PathBuf::from("//./pipe/Gnosis VPN")
// }
