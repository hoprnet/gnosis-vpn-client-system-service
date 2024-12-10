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
        Err(CtrlcError::System(err)) => {
            tracing::error!(%err, "system error");
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
        Err(err) => {
            tracing::error!(%err, "error checking socket path");
            return Err(exitcode::IOERR);
        }
    };

    let stream = match net::UnixListener::bind(socket_path) {
        Ok(listener) => listener,
        Err(err) => {
            tracing::error!(%err, "error binding socket");
            return Err(exitcode::OSFILE);
        }
    };

    // update permissions to allow unprivileged access
    // TODO this would better be handled by allowing group access and let the installer create a
    // gvpn group and additionally add users to it
    match fs::set_permissions(socket_path, fs::Permissions::from_mode(0o666)) {
        Ok(_) => (),
        Err(err) => {
            tracing::error!(%err, "error setting socket permissions");
            return Err(exitcode::NOPERM);
        }
    }

    let (sender, receiver) = crossbeam_channel::unbounded::<net::UnixStream>();
    thread::spawn(move || {
        for strm in stream.incoming() {
            _ = match strm {
                Ok(s) => match sender.send(s) {
                    Ok(_) => (),
                    Err(err) => {
                        tracing::error!(%err, "sending incoming data");
                    }
                },
                Err(err) => {
                    tracing::error!(%err, "waiting for incoming message");
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
        Err(err) => {
            tracing::error!(%err, "error receiving stream");
            return;
        }
    };

    let mut msg = String::new();
    if let Err(err) = stream.read_to_string(&mut msg) {
        tracing::error!(%err, "error reading message");
        return;
    };

    let cmd = match msg.parse::<Command>() {
        Ok(cmd) => cmd,
        Err(err) => {
            tracing::error!(%err, %msg, "error parsing command");
            return;
        }
    };
    tracing::debug!(%cmd, "parsed command");

    let res = match state.handle_cmd(&cmd) {
        Ok(res) => res,
        Err(err) => {
            tracing::error!(%err, "error handling command");
            return;
        }
    };

    if let Some(resp) = res {
        tracing::info!(response = %resp);
        if let Err(err) = stream.write_all(resp.as_bytes()) {
            tracing::error!(%err, "error writing response");
            return;
        }
        if let Err(err) = stream.flush() {
            tracing::error!(%err, "error flushing stream");
            return;
        }
    }
}

#[tracing::instrument(skip(res_event), level = Level::DEBUG) ]
fn incoming_event(state: &mut core::Core, res_event: Result<event::Event, crossbeam_channel::RecvError>) -> () {
    let event: event::Event = match res_event {
        Ok(evt) => evt,
        Err(err) => {
            tracing::error!(%err, "error receiving event");
            return;
        }
    };

    match state.handle_event(event) {
        Ok(_) => (),
        Err(err) => {
            tracing::error!(%err, "error handling event");
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
        Err(err) => tracing::warn!(%err, "failed removing socket"),
    }
    process::exit(exit)
}
