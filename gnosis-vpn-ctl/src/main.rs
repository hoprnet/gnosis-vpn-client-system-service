use anyhow::{anyhow, Context};
use clap::{Parser, Subcommand};
use std::io::{Read, Write};
use std::os::unix::net;
use std::path::Path;

/// Gnosis VPN system service - offers interaction commands on Gnosis VPN to other applications.
#[derive(Parser)]
struct Cli {
    /// communication socket name - will be created by installer and should be /run/gnosisvpn/service.sock
    #[arg(short, long)]
    socket: String,

    /// command to run
    #[command(subcommand)]
    cmd: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    // Simple status command
    Status,
    // Connect to wg server
//     WgConnect {
//         #[arg(short, long)]
//         peer: String,
//         #[arg(short, long)]
//         allowed_ips: String,
//         #[arg(short, long)]
//         endpoint: String,
//     },
}

fn run_command(socket: &String, cmd: Commands) -> anyhow::Result<()> {
    let res = Path::try_exists(Path::new(socket));

    let typed_cmd = match cmd {
        Commands::Status => gnosis_vpn_lib::Command::Status,
        // Commands::WgConnect {
        //     peer,
        //     allowed_ips,
        //     endpoint,
        // } => gnosis_vpn_lib::Command::WgConnect {
        //     peer,
        //     allowed_ips,
        //     endpoint,
        // },
    };

    let str_cmd = gnosis_vpn_lib::to_string(&typed_cmd)?;

    let mut sender = match res {
        Ok(true) => net::UnixStream::connect(socket).with_context(|| "unable to connect to socket"),
        Ok(false) => Err(anyhow!(format!("gnosis-vpn not running"))),
        Err(x) => Err(anyhow!(x)),
    }?;

    log::info!("sending command: {}", str_cmd);
    sender.write_all(str_cmd.as_bytes())?;
    sender.flush()?;
    handle_response(typed_cmd, sender)?;
    Ok(())
}

fn handle_response(cmd: gnosis_vpn_lib::Command, mut sender: net::UnixStream) -> anyhow::Result<()> {
    match cmd {
        gnosis_vpn_lib::Command::Status => {
            let mut response = String::new();
            sender.read_to_string(&mut response)?;
            log::info!("response: {}", response);
            Ok(())
        }

    }
}

fn main() {
    env_logger::init();
    let args = Cli::parse();
    let res = args.cmd.map(|c| run_command(&args.socket, c));
    match res {
        Some(Ok(_)) => log::info!("stopped gracefully"),
        Some(Err(x)) => log::error!("stopped with error: {}", x),
        None => log::info!("no command specified"),
    }
}
