use clap::Parser;
use gnosis_vpn_lib::command::Command;
use gnosis_vpn_lib::socket;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net;
use std::path::Path;
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

fn ctrl_channel() -> anyhow::Result<crossbeam_channel::Receiver<()>> {
    let (sender, receiver) = crossbeam_channel::bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

#[tracing::instrument(skip(res_stream), level = Level::DEBUG) ]
fn incoming_stream(state: &mut core::Core, res_stream: Result<net::UnixStream, std::io::Error>) -> () {
    let mut stream: net::UnixStream = match res_stream {
        Ok(stream) => stream,
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

fn incoming_event(event: Result<event::Event, crossbeam_channel::RecvError>) {
    match event {
        Ok(evt) => {
            let res = state.handle_event(evt);
            foo
        }
        Err(err) => (),
    }
}

fn daemon(socket_path: &Path) -> anyhow::Result<()> {
    let ctrl_c_events = ctrl_channel()?;

    let res_exists = socket_path.try_exists();

    // set up unix stream listener
    let listener = match res_exists {
        Ok(true) => Err(anyhow!(format!("already running"))),
        Ok(false) => net::UnixListener::bind(socket_path).context("failed to bind socket"),
        Err(err) => Err(anyhow!(err)),
    }?;

    // update permissions to allow unprivileged access
    // TODO this would better be handled by allowing group access and let the installer create a
    // gvpn group and additionally add users to it
    fs::set_permissions(socket_path, fs::Permissions::from_mode(0o666))?;

    let (sender_socket, receiver_socket) = crossbeam_channel::unbounded::<net::UnixStream>();
    thread::spawn(move || {
        for stream in listener.incoming() {
            _ = match stream {
                Ok(stream) => sender_socket.send(stream).context("failed to send stream to channel"),
                Err(err) => {
                    tracing::error!(%err, "waiting for incoming message");
                    Err(anyhow!(err))
                }
            };
        }
    });

    let (sender_core_loop, receiver_core_loop) = crossbeam_channel::unbounded::<event::Event>();
    let mut state = core::Core::init(sender_core_loop);
    tracing::info!("started in listening mode");
    loop {
        crossbeam_channel::select! {
            recv(ctrl_c_events) -> _ => {
                tracing::info!("shutting down");
                break;
            }
            recv(receiver_socket) -> stream => incoming_stream(&mut stream),
            recv(receiver_core_loop) -> event => incoming_event(event),
        }
    }
    Ok(())
}

fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let _args = Cli::parse();
    let socket_path = socket::socket_path();

    // run continously until ctrl-c
    match daemon(&socket_path) {
        Ok(_) => (),
        Err(e) => {
            // Log the error and its chain in one line
            let error_chain: Vec<String> = e.chain().map(|cause| cause.to_string()).collect();
            tracing::error!(?error_chain, "Exiting with error");
        }
    }

    // cleanup
    match fs::remove_file(socket_path) {
        Ok(_) => tracing::info!("stopped gracefully"),
        Err(err) => tracing::warn!(%err, "failed removing socket"),
    }
}
