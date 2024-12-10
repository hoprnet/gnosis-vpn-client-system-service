use clap::Parser;
use ctrlc::Error as CtrlcError;
use gnosis_vpn_lib::command::Command;
use gnosis_vpn_lib::socket;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net;
use std::path::Path;
use std::process;
use std::thread;
use tracing::Level;

mod backoff;
mod core;
mod entry_node;
mod event;
mod exit_node;
mod remote_data;
mod session;

use crate::core::error::Error as CoreError;

/// Gnosis VPN system service - offers interaction commands on Gnosis VPN to other applications.
#[derive(Parser)]
struct Cli {}

#[tracing::instrument(level = Level::DEBUG)]
fn ctrl_channel() -> Result<crossbeam_channel::Receiver<()>, exitcode::ExitCode> {
    let (sender, receiver) = crossbeam_channel::bounded(100);
    match ctrlc::set_handler(move || {
        let _ = sender.send(());
    }) {
        Ok(_) => Ok(receiver),
        Err(CtrlcError::NoSuchSignal(signal_type)) => {
            tracing::error!(?signal_type, "no such signal");
            Err(exitcode::OSERR)
        }
        Err(CtrlcError::MultipleHandlers) => {
            tracing::error!("multiple handlers");
            Err(exitcode::UNAVAILABLE)
        }
        Err(CtrlcError::System(e)) => {
            tracing::error!(error = ?e, "system error");
            Err(exitcode::IOERR)
        }
    }
}

#[tracing::instrument(level = Level::DEBUG)]
fn socket_channel(socket_path: &Path) -> Result<crossbeam_channel::Receiver<net::UnixStream>, exitcode::ExitCode> {
    match socket_path.try_exists() {
        Ok(true) => {
            tracing::error!("socket path already exists");
            return Err(exitcode::TEMPFAIL);
        }
        Ok(false) => (),
        Err(e) => {
            tracing::error!(error = ?e, "error checking socket path");
            return Err(exitcode::IOERR);
        }
    };

    let stream = match net::UnixListener::bind(socket_path) {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!(error = ?e, "error binding socket");
            return Err(exitcode::OSFILE);
        }
    };

    // update permissions to allow unprivileged access
    // TODO this would better be handled by allowing group access and let the installer create a
    // gvpn group and additionally add users to it
    match fs::set_permissions(socket_path, fs::Permissions::from_mode(0o666)) {
        Ok(_) => (),
        Err(e) => {
            tracing::error!(error = ?e, "error setting socket permissions");
            return Err(exitcode::NOPERM);
        }
    }

    let (sender, receiver) = crossbeam_channel::unbounded::<net::UnixStream>();
    thread::spawn(move || {
        for strm in stream.incoming() {
            match strm {
                Ok(s) => match sender.send(s) {
                    Ok(_) => (),
                    Err(e) => {
                        tracing::error!(error = ?e, "sending incoming data");
                    }
                },
                Err(e) => {
                    tracing::error!(error = ?e, "waiting for incoming message");
                }
            };
        }
    });

    Ok(receiver)
}

#[tracing::instrument(skip(res_stream), level = Level::DEBUG) ]
fn incoming_stream(state: &mut core::Core, res_stream: Result<net::UnixStream, crossbeam_channel::RecvError>) -> () {
    let mut stream: net::UnixStream = match res_stream {
        Ok(strm) => strm,
        Err(e) => {
            tracing::error!(error = ?e, "error receiving stream");
            return;
        }
    };

    let mut msg = String::new();
    if let Err(e) = stream.read_to_string(&mut msg) {
        tracing::error!(error = ?e, "error reading message");
        return;
    };

    let cmd = match msg.parse::<Command>() {
        Ok(cmd) => cmd,
        Err(e) => {
            tracing::error!(error = ?e, %msg, "error parsing command");
            return;
        }
    };
    tracing::debug!(command = %cmd, "parsed command");

    let res = match state.handle_cmd(&cmd) {
        Ok(res) => res,
        Err(CoreError::ParseJson(e)) => {
            tracing::error!(error = ?e, "error parsing json");
            return;
        }
        Err(CoreError::UnexpectedInternalState(deviation)) => {
            tracing::error!(?deviation, "unexpected internal state");
            return;
        }
        Err(CoreError::HeaderSerialization(e)) => {
            tracing::error!(error = ?e, "invalid header value");
            return;
        }
        Err(CoreError::Url(e)) => {
            tracing::error!(error = ?e, "error parsing url");
            return;
        }
    };

    if let Some(resp) = res {
        tracing::info!(response = %resp);
        if let Err(e) = stream.write_all(resp.as_bytes()) {
            tracing::error!(error = ?e, "error writing response");
            return;
        }
        if let Err(e) = stream.flush() {
            tracing::error!(error = ?e, "error flushing stream");
            return;
        }
    }
}

#[tracing::instrument(skip(res_event), level = Level::DEBUG) ]
fn incoming_event(state: &mut core::Core, res_event: Result<event::Event, crossbeam_channel::RecvError>) -> () {
    let event: event::Event = match res_event {
        Ok(evt) => evt,
        Err(e) => {
            tracing::error!(error = ?e, "error receiving event");
            return;
        }
    };

    match state.handle_event(event) {
        Ok(_) => (),
        Err(e) => {
            tracing::error!(error = ?e, "error handling event");
            return;
        }
    }
}

fn daemon(socket_path: &Path) -> exitcode::ExitCode {
    let ctrlc_receiver = match ctrl_channel() {
        Ok(receiver) => receiver,
        Err(exit) => return exit,
    };

    let socket_receiver = match socket_channel(socket_path) {
        Ok(receiver) => receiver,
        Err(exit) => return exit,
    };

    let (sender, core_receiver) = crossbeam_channel::unbounded::<event::Event>();
    let mut state = core::Core::init(sender);

    tracing::info!("started in listening mode");
    loop {
        crossbeam_channel::select! {
            recv(ctrlc_receiver) -> _ => {
                tracing::info!("shutting down");
                return exitcode::OK;
            }
            recv(socket_receiver) -> stream => incoming_stream(&mut state, stream),
            recv(core_receiver) -> event => incoming_event(&mut state, event),
        }
    }
}

fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let _args = Cli::parse();
    let socket_path = socket::socket_path();

    // run continously until ctrl-c
    let exit = daemon(&socket_path);

    // cleanup
    match fs::remove_file(socket_path) {
        Ok(_) => tracing::info!("stopped gracefully"),
        Err(e) => tracing::warn!(error = %e, "failed removing socket"),
    }
    process::exit(exit)
}
