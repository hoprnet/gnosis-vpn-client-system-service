use clap::Parser;
use std::io;
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

fn run_command(socket: &String, cmd: &String) -> Result<(), io::Error> {
    let res = Path::try_exists(Path::new(socket));

    let mut sender = match res {
        Ok(true) => net::UnixStream::connect(socket),
        Ok(false) => Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Daemon not running",
        )),
        Err(x) => Err(x),
    }?;

    return sender.write_all(cmd.as_bytes());
}

fn main() {
    env_logger::init();
    let args = Cli::parse();
    let res = run_command(&args.socket, &args.cmd);
    log::info!("patthern: {:?}, result: {:?}", args.socket, res);
}
