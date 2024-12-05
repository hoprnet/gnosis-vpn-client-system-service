use anyhow::{anyhow, Context};
use clap::Parser;
use gnosis_vpn_lib::command::Command;
use gnosis_vpn_lib::socket;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net;
use std::path::PathBuf;
use std::thread;

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

fn incoming_stream(stream: &mut net::UnixStream) -> anyhow::Result<Command> {
    let mut incoming = String::new();
    stream.read_to_string(&mut incoming)?;
    incoming
        .parse::<Command>()
        .with_context(|| format!("error parsing incoming stream: {}", incoming))
}

fn respond_stream(stream: &mut net::UnixStream, res: Option<String>) -> anyhow::Result<()> {
    if let Some(resp) = res {
        tracing::info!("responding: {}", resp);
        stream.write_all(resp.as_bytes())?;
        stream.flush()?;
    }
    Ok(())
}

fn daemon(socket_path: PathBuf) -> anyhow::Result<()> {
    let ctrl_c_events = ctrl_channel()?;

    let res_exists = socket_path.try_exists();

    // set up unix stream listener
    let listener = match res_exists {
        Ok(true) => Err(anyhow!(format!("already running"))),
        Ok(false) => net::UnixListener::bind(socket_path.as_path()).context("failed to bind socket"),
        Err(x) => Err(anyhow!(x)),
    }?;

    // update permissions to allow unprivileged access
    // TODO this would better be handled by allowing group access and let the installer create a
    // gvpn group and additionally add users to it
    fs::set_permissions(socket_path.as_path(), fs::Permissions::from_mode(0o666))?;

    let (sender_socket, receiver_socket) = crossbeam_channel::unbounded::<net::UnixStream>();
    thread::spawn(move || {
        for stream in listener.incoming() {
            _ = match stream {
                Ok(stream) => sender_socket.send(stream).context("failed to send stream to channel"),
                Err(x) => {
                    tracing::error!("error waiting for incoming message: {:?}", x);
                    Err(anyhow!(x))
                }
            };
        }
    });

    let (sender_core_loop, receiver_core_loop) = crossbeam_channel::unbounded::<event::Event>();
    let mut state = core::Core::init(sender_core_loop);
    tracing::info!("started successfully in listening mode");
    loop {
        crossbeam_channel::select! {
            recv(ctrl_c_events) -> _ => {
                tracing::info!("shutting down");
                break;
            }
            recv(receiver_socket) -> stream => {
                let res = match stream  {
                    Ok(mut s) =>
                        incoming_stream(&mut s)
                            .and_then(|cmd| state.handle_cmd(cmd))
                            .and_then(|res| respond_stream(&mut s, res)),
                    Err(x) => Err(anyhow!(x))
                };
                if let Err(x) = res {
                    tracing::error!("error handling incoming stream: {:?}", x);
                }
            },
            recv(receiver_core_loop) -> event => {
                let res = match event {
                    Ok(evt) => state.handle_event(evt),
                    Err(x) => Err(anyhow!(x))
                };
                if let Err(x) = res {
                    tracing::error!("error handling event: {:?}", x);
                }
            }
        }
    }

    fs::remove_file(socket_path)?;
    Ok(())
}

fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let _args = Cli::parse();
    let socket_path = socket::socket_path();
    let res = daemon(socket_path);
    match res {
        Ok(_) => tracing::info!("stopped gracefully"),
        Err(e) => {
            // Log the error and its chain in one line
            let error_chain: Vec<String> = e.chain().map(|cause| cause.to_string()).collect();
            tracing::error!(?error_chain, "Exiting with error");
        }
    }
}
