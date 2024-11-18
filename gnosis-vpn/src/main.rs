use anyhow::{anyhow, Context};
use clap::Parser;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net;
use std::path::Path;
use std::thread;

mod core;

/// Gnosis VPN system service - offers interaction commands on Gnosis VPN to other applications.
#[derive(Parser)]
struct Cli {
    /// communication socket name - will be created by installer and should be /run/gnosisvpn/service.sock
    #[arg(short, long)]
    socket: String,
}

fn ctrl_channel() -> anyhow::Result<crossbeam_channel::Receiver<()>> {
    let (sender, receiver) = crossbeam_channel::bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

fn incoming_stream(state: &mut core::Core, mut stream: net::UnixStream) -> anyhow::Result<()> {
    let mut buffer = [0; 128];
    let size = stream.read(&mut buffer)?;
    let inc = String::from_utf8_lossy(&buffer[..size]);
    log::info!("incoming: {}", inc);
    let cmd = gnosis_vpn_lib::to_cmd(inc.as_ref())?;
    incoming(state, stream, cmd)
}

fn incoming(
    state: &mut core::Core,
    mut stream: net::UnixStream,
    cmd: gnosis_vpn_lib::Command,
) -> anyhow::Result<()> {
    let res = match cmd {
        gnosis_vpn_lib::Command::Status => state.status(),
        gnosis_vpn_lib::Command::EntryNode {
            endpoint,
            api_token,
        } => state.entry_node(endpoint, api_token),
    }?;

    if let Some(resp) = res {
        stream
            .write_all(resp.as_bytes())
            .with_context(|| "failed to write response")?;
        stream.flush().with_context(|| "failed to flush response")?
    };
    Ok(())
}

fn daemon(state: &mut core::Core, socket: &String) -> anyhow::Result<()> {
    let ctrl_c_events = ctrl_channel()?;

    let socket_path = Path::new(socket);
    let res_exists = Path::try_exists(socket_path);

    let listener = match res_exists {
        Ok(true) => Err(anyhow!(format!("already running"))),
        Ok(false) => net::UnixListener::bind(socket)
            .with_context(|| format!("error binding listener to socket {}", socket)),
        Err(x) => Err(anyhow!(x)),
    }?;

    let (sender, receiver) = crossbeam_channel::unbounded::<net::UnixStream>();
    thread::spawn(move || {
        for stream in listener.incoming() {
            _ = match stream {
                Ok(stream) => sender
                    .send(stream)
                    .with_context(|| "failed to send stream to channel"),
                Err(x) => {
                    log::error!("error waiting for incoming message: {:?}", x);
                    Err(anyhow!(x))
                }
            };
        }
    });

    state.started();
    log::info!("started successfully in listening mode");
    loop {
        crossbeam_channel::select! {
            recv(ctrl_c_events) -> _ => {
                log::info!("shutting down");
                break;
            }
            recv(receiver) -> stream => {
                _ = match stream  {
                    Ok(s) => incoming_stream(state, s),
                    Err(x) => Err(anyhow!(x))

                }
            },
        }
    }

    fs::remove_file(socket_path)?;
    Ok(())
}

fn main() {
    env_logger::init();
    let args = Cli::parse();
    let mut state = core::Core::init();
    let res = daemon(&mut state, &args.socket);
    match res {
        Ok(_) => log::info!("stopped gracefully"),
        Err(x) => log::error!("stopped with error: {}", x),
    }
}
