use anyhow::{anyhow, Context};
use clap::Parser;
use std::fs;
use std::io::Read;
use std::os::unix::net;
use std::path::Path;
use std::thread;

const DAEMON: &str = "GnosisVPN daemon";

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

fn incoming(mut stream: net::UnixStream) -> anyhow::Result<()> {
    let mut buffer = [0; 128];
    let size = stream.read(&mut buffer)?;
    log::info!("incoming: {}", String::from_utf8_lossy(&buffer[..size]));
    Ok(())
}

fn daemon(socket: &String) -> anyhow::Result<()> {
    let ctrl_c_events = ctrl_channel()?;

    let socket_path = Path::new(socket);
    let res_exists = Path::try_exists(socket_path);

    let listener = match res_exists {
        Ok(true) => Err(anyhow!(format!("{DAEMON} already running"))),
        Ok(false) => net::UnixListener::bind(socket)
            .with_context(|| format!("Error binding listener to socket {}", socket)),
        Err(x) => Err(anyhow!(x)),
    }?;

    let (sender, receiver) = crossbeam_channel::unbounded::<net::UnixStream>();

    let sender = sender.clone();
    thread::spawn(move || {
        for stream in listener.incoming() {
            _ = match stream {
                Ok(stream) => sender
                    .send(stream)
                    .with_context(|| format!("Failed to send stream to channel")),
                Err(x) => {
                    log::error!("Error waiting for incoming message: {:?}", x);
                    Err(anyhow!(x))
                }
            };
        }
    });

    log::info!("{DAEMON} started successfully in listening mode");
    loop {
        crossbeam_channel::select! {
            recv(ctrl_c_events) -> _ => {
                log::info!("{DAEMON} shutting down");
                break;
            }
            recv(receiver) -> stream => {
                _ = match stream  {
                    Ok(s) => {
                incoming(s)
                    },
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
    daemon(&args.socket).expect(&format!("{DAEMON} exited with error"));
}
