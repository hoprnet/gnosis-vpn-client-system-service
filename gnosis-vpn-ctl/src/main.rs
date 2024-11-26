use anyhow::{anyhow, Context};
use std::{
    io::{Read, Write},
    matches,
    os::unix::net,
    path::Path,
};
use tracing::{debug, info, instrument};

use gnosis_vpn_ctl::{cli, Command};

fn as_internal_cmd(cmd: Command) -> gnosis_vpn_lib::Command {
    match cmd {
        Command::Status => gnosis_vpn_lib::Command::Status,
        Command::EntryNode { endpoint, api_token } => gnosis_vpn_lib::Command::EntryNode { endpoint, api_token },
        Command::ExitNode { peer_id } => gnosis_vpn_lib::Command::ExitNode {
            peer_id: peer_id.to_string(),
        },
    }
}

#[instrument(level = "debug", ret(Debug))]
fn execute_internal_command(
    socket: &mut net::UnixStream,
    cmd: gnosis_vpn_lib::Command,
) -> anyhow::Result<Option<String>> {
    socket.write_all(format!("{cmd}").as_bytes())?;
    socket.flush()?;

    Ok(if matches!(cmd, gnosis_vpn_lib::Command::Status) {
        let mut response = String::new();
        socket.read_to_string(&mut response)?;
        Some(response)
    } else {
        None
    })
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let options = cli().run();

    debug!(?options, "Options parsed");

    let socket = options.socket;

    for cmd in options.commands.into_iter() {
        let mut socket = match Path::try_exists(Path::new(&socket)) {
            Ok(true) => net::UnixStream::connect(socket.clone()).with_context(|| "unable to connect to socket"),
            Ok(false) => Err(anyhow!(format!("gnosis-vpn not running"))),
            Err(x) => Err(anyhow!(x)),
        }?;

        debug!(?socket, "Socket connected");

        if let Some(response) = execute_internal_command(&mut socket, as_internal_cmd(cmd))? {
            info!(%response, "Command result");
        }
    }

    info!("Commands executed successfully");

    Ok(())
}
