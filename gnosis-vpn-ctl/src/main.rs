use anyhow::{anyhow, Context};
use std::io::Write;
use clap::Parser;
use std::os::unix::net;
use std::path::Path;

const APP: &str = "GnosisVPN control client";
const DAEMON: &str = "GnosisVPN daemon";

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
        Ok(true) => net::UnixStream::connect(socket).with_context(|| format!("{APP} unable to connect to socket")),
        Ok(false) => Err(anyhow!(format!("{DAEMON} not running"))),
        Err(x) => Err(anyhow!(x)),
    }?;

    log::info!("{APP} sending command: {}", cmd);
    sender.write_all(cmd.as_bytes())?;
    Ok(())
}

fn main() {
    env_logger::init();
    let args = Cli::parse();
    run_command(&args.socket, &args.cmd).expect(&format!("{APP} exited with error"));
}
