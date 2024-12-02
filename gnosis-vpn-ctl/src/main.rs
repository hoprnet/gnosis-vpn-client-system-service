use anyhow::{anyhow, Context};
use std::{
    io::{Read, Write},
    os::unix::net,
};
use tracing::{debug, info, instrument};

use gnosis_vpn_ctl::{cli, Command};

fn as_internal_cmd(cmd: &Command) -> gnosis_vpn_lib::Command {
    match cmd {
        Command::Status => gnosis_vpn_lib::Command::Status,
        Command::EntryNode {
            endpoint,
            api_token,
            listen_host,
        } => gnosis_vpn_lib::Command::EntryNode {
            endpoint: endpoint.clone(),
            api_token: api_token.clone(),
            listen_host: listen_host.clone(),
        },
        Command::ExitNode { peer_id } => gnosis_vpn_lib::Command::ExitNode { peer_id: *peer_id },
    }
}

#[instrument(level = "debug", ret(Debug))]
fn execute_internal_command(socket: &mut net::UnixStream, cmd: gnosis_vpn_lib::Command) -> anyhow::Result<()> {
    let cmd_as_json = cmd.to_json_string()?;
    socket.write_all(cmd_as_json.as_bytes())?;
    socket.flush().context("unable to flush socket")
}

fn main() -> anyhow::Result<()> {
    let log_folder = std::env::var_os("GPVN_CTL_LOG_FOLDER").unwrap_or(".".into());
    let file_appender = tracing_appender::rolling::minutely(log_folder, "gnosisvpn-ctl.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).init();

    let options = cli().run();

    debug!(?options, "Options parsed");

    let socket_path = &gnosis_vpn_lib::socket_path();

    for cmd in options.commands.into_iter() {
        let mut socket = match socket_path.try_exists() {
            Ok(true) => net::UnixStream::connect(socket_path).context("unable to connect to socket"),
            Ok(false) => Err(anyhow!(format!("gnosis-vpn not running"))),
            Err(x) => Err(anyhow!(x)),
        }?;

        debug!(?socket, "Socket connected");

        execute_internal_command(&mut socket, as_internal_cmd(&cmd))?;

        if let Command::Status = cmd {
            // Shutdown the write operation to signal the other side command has been sent
            // This will force EOF thus allowing the other side to process the command
            socket.shutdown(std::net::Shutdown::Write)?;
            let mut response = String::new();
            socket.read_to_string(&mut response)?;
            info!(%response, "Command result");
            println!("{}", response);
        }
    }

    info!("Commands executed successfully");

    Ok(())
}
