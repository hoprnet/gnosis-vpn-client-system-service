use clap::Parser;
use ctrlc::Error as CtrlcError;
use gnosis_vpn_lib::command::Command;
use gnosis_vpn_lib::config;
use gnosis_vpn_lib::socket;
use notify::{RecursiveMode, Watcher};
use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net;
use std::path::Path;
use std::process;
use std::thread;
use std::time::{Duration, Instant};
use tracing::Level;

mod backoff;
mod core;
mod entry_node;
mod event;
mod exit_node;
mod remote_data;
mod session;
mod task;
mod wireguard;

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
fn config_channel() -> Result<
    (
        notify::RecommendedWatcher,
        crossbeam_channel::Receiver<notify::Result<notify::Event>>,
    ),
    exitcode::ExitCode,
> {
    let (sender, receiver) = crossbeam_channel::unbounded::<notify::Result<notify::Event>>();

    let path = config::path();
    let parent = match path.parent() {
        Some(dir) => dir,
        None => {
            tracing::error!("config path has no parent");
            return Err(exitcode::UNAVAILABLE);
        }
    };

    if !parent.exists() {
        match fs::create_dir(parent) {
            Ok(_) => (),
            Err(e) => {
                tracing::error!(error = ?e, "error creating config directory");
                return Err(exitcode::IOERR);
            }
        }
    }

    let mut watcher = match notify::recommended_watcher(sender) {
        Ok(watcher) => watcher,
        Err(e) => {
            tracing::error!(error = ?e, "error creating config watcher");
            return Err(exitcode::IOERR);
        }
    };

    match watcher.watch(parent, RecursiveMode::NonRecursive) {
        Ok(_) => (),
        Err(e) => {
            tracing::error!(error = ?e, "error watching config directory");
            return Err(exitcode::IOERR);
        }
    };

    Ok((watcher, receiver))
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
        Err(e) => {
            // Log the error and its chain in one line
            let error_chain: Vec<String> = e.chain().map(|cause| cause.to_string()).collect();
            tracing::error!(?error_chain, "error handling command");
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
            // Log the error and its chain in one line
            let error_chain: Vec<String> = e.chain().map(|cause| cause.to_string()).collect();
            tracing::error!(?error_chain, "error handling event");
            return;
        }
    }
}

// handling fs config events with a grace period to avoid duplicate reads without delay
const CONFIG_GRACE_PERIOD: Duration = Duration::from_millis(333);

#[tracing::instrument(skip(res_event), level = Level::DEBUG) ]
fn incoming_config_fs_event(
    res_event: Result<notify::Result<notify::Event>, crossbeam_channel::RecvError>,
) -> Option<crossbeam_channel::Receiver<Instant>> {
    let event: notify::Result<notify::Event> = match res_event {
        Ok(evt) => evt,
        Err(e) => {
            tracing::error!(error = ?e, "error receiving config event");
            return None;
        }
    };

    match event {
        Ok(notify::Event { kind, paths, attrs: _ })
            if kind == notify::event::EventKind::Create(notify::event::CreateKind::File)
                && paths == vec![config::path()] =>
        {
            tracing::debug!("config file created");
            Some(crossbeam_channel::after(CONFIG_GRACE_PERIOD))
        }
        Ok(notify::Event { kind, paths, attrs: _ })
            if kind
                == notify::event::EventKind::Modify(notify::event::ModifyKind::Data(
                    notify::event::DataChange::Any,
                ))
                && paths == vec![config::path()] =>
        {
            tracing::debug!("config file modified");
            Some(crossbeam_channel::after(CONFIG_GRACE_PERIOD))
        }
        Ok(notify::Event { kind, paths, attrs: _ })
            if kind == notify::event::EventKind::Remove(notify::event::RemoveKind::File)
                && paths == vec![config::path()] =>
        {
            tracing::debug!("config file removed");
            Some(crossbeam_channel::after(CONFIG_GRACE_PERIOD))
        }
        Ok(_) => None,
        Err(e) => {
            tracing::error!(error = ?e, "error watching config folder");
            None
        }
    }
}

fn daemon(socket_path: &Path) -> exitcode::ExitCode {
    let ctrlc_receiver = match ctrl_channel() {
        Ok(receiver) => receiver,
        Err(exit) => return exit,
    };

    // keep config watcher in scope so it does not get dropped
    let (_config_watcher, config_receiver) = match config_channel() {
        Ok(receiver) => receiver,
        Err(exit) => return exit,
    };

    let socket_receiver = match socket_channel(socket_path) {
        Ok(receiver) => receiver,
        Err(exit) => return exit,
    };

    let (sender, core_receiver) = crossbeam_channel::unbounded::<event::Event>();
    let mut state = core::Core::init(sender);

    let mut read_config_receiver: crossbeam_channel::Receiver<Instant> = crossbeam_channel::never();

    tracing::info!("started in listening mode");
    loop {
        crossbeam_channel::select! {
            recv(ctrlc_receiver) -> _ => {
                tracing::info!("shutting down");
                return exitcode::OK;
            }
            recv(socket_receiver) -> stream => incoming_stream(&mut state, stream),
            recv(core_receiver) -> event => incoming_event(&mut state, event),
            recv(config_receiver) -> event => {
                incoming_config_fs_event(event).map(|r| read_config_receiver = r);
            },
            recv(read_config_receiver) -> _ => state.update_config(),
        }
    }
}

fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let _args = Cli::parse();
    let socket_path = socket::path();

    // run continously until ctrl-c
    let exit = daemon(&socket_path);

    // cleanup
    match fs::remove_file(socket_path) {
        Ok(_) => tracing::info!("stopped gracefully"),
        Err(e) => tracing::warn!(error = %e, "failed removing socket"),
    }
    process::exit(exit)
}
