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

fn incoming_stream(stream: &mut net::UnixStream) -> anyhow::Result<gnosis_vpn_lib::Command> {
    let mut buffer = [0; 128];
    let size = stream.read(&mut buffer)?;
    let inc = String::from_utf8_lossy(&buffer[..size]);
    log::info!("incoming: {}", inc);
    gnosis_vpn_lib::to_cmd(inc.as_ref())
}

fn respond_stream(stream: &mut net::UnixStream, res: Option<String>) -> anyhow::Result<()> {
    if let Some(resp) = res {
        log::info!("responding: {}", resp);
    stream.write_all(resp.as_bytes())?;
    stream.flush()?;
    }
    Ok(())
}

fn daemon(socket: &String) -> anyhow::Result<()> {
    let ctrl_c_events = ctrl_channel()?;

    let socket_path = Path::new(socket);
    let res_exists = Path::try_exists(socket_path);

    // set up unix stream listener
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

    let mut state = core::Core::init();
    log::info!("started successfully in listening mode");
    loop {
        crossbeam_channel::select! {
            recv(ctrl_c_events) -> _ => {
                log::info!("shutting down");
                break;
            }
            recv(receiver) -> stream => {
                let res = match stream  {
                    Ok(mut s) =>
                        incoming_stream(&mut s)
                            .and_then(|cmd| state.handle_cmd(cmd))
                            .and_then(|res| respond_stream(&mut s, res)),
                    Err(x) => Err(anyhow!(x))
                };
                if let Err(x) = res {
                    log::error!("error handling incoming stream: {:?}", x);
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
    let res = daemon(&args.socket);
    match res {
        Ok(_) => log::info!("stopped gracefully"),
        Err(x) => log::error!("stopped with error: {}", x),
    }
}
