use anyhow::{anyhow, Context};
use clap::{Parser, Subcommand};
use std::io::{Read, Write};
use std::os::unix::net;
use std::path::Path;
use url::Url;

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

    EntryNode {
        #[arg(short, long)]
        endpoint: String,
        #[arg(short, long)]
        api_token: String,
    },

    ExitNode {
        // libp2p_identity::PeerId
        #[arg(short, long)]
        peer_id: String,
    },
}

fn run_command(socket: &String, cmd: Commands) -> anyhow::Result<()> {
    let res = Path::try_exists(Path::new(socket));

    let typed_cmd = match cmd {
        Commands::Status => gnosis_vpn_lib::Command::Status,
        Commands::EntryNode {
            endpoint,
            api_token,
        } => gnosis_vpn_lib::Command::EntryNode {
            endpoint: Url::parse(&endpoint).with_context(|| "invalid endpoint URL")?,
            api_token,
        },
        Commands::ExitNode { peer_id } => gnosis_vpn_lib::Command::ExitNode { peer_id },
    };

    let mut sender = match res {
        Ok(true) => net::UnixStream::connect(socket).with_context(|| "unable to connect to socket"),
        Ok(false) => Err(anyhow!(format!("gnosis-vpn not running"))),
        Err(x) => Err(anyhow!(x)),
    }?;

    tracing::info!("sending command: {}", typed_cmd);
    sender.write_all(format!("{typed_cmd}").as_bytes())?;
    sender.flush()?;
    handle_response(typed_cmd, sender)?;
    Ok(())
}

fn handle_response(
    cmd: gnosis_vpn_lib::Command,
    mut sender: net::UnixStream,
) -> anyhow::Result<()> {
    // handle responses only for certain commands
    match cmd {
        gnosis_vpn_lib::Command::Status => {
            let mut response = String::new();
            sender.read_to_string(&mut response)?;
            tracing::info!("response: {}", response);
            Ok(())
        }
        _ => Ok(()),
    }
}

fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let args = Cli::parse();
    let res = args.cmd.map(|c| run_command(&args.socket, c));
    match res {
        Some(Ok(_)) => tracing::info!("stopped gracefully"),
        Some(Err(x)) => tracing::error!("stopped with error: {}", x),
        None => tracing::info!("no command specified"),
    }
}
