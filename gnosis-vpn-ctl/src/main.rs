use anyhow::{anyhow, Context};
use clap::Parser;
use std::io::Write;
use std::os::unix::net;
use std::path::Path;

/// Gnosis VPN system service - offers interaction commands on Gnosis VPN to other applications.
#[derive(Parser)]
struct Cli {
    /// communication socket name - will be created by installer and should be /run/gnosisvpn/service.sock
    #[arg(short, long)]
    socket: String,

    /// command to run
    cmd: String,
}

fn run_command(socket: &String, cmd: &String) -> anyhow::Result<()> {
    let res = Path::try_exists(Path::new(socket));

    let mut sender = match res {
        Ok(true) => {
            net::UnixStream::connect(socket).with_context(|| format!("unable to connect to socket"))
        }
        Ok(false) => Err(anyhow!(format!("gnosis-vpn not running"))),
        Err(x) => Err(anyhow!(x)),
    }?;

    log::info!("sending command: {}", cmd);
    sender.write_all(cmd.as_bytes())?;
    Ok(())
}

fn main() {
    env_logger::init();
    let args = Cli::parse();
    let res = run_command(&args.socket, &args.cmd);
    match res {
        Ok(_) => log::info!("stopped gracefully"),
        Err(x) => log::error!("stopped with error: {}", x),
    }
}
