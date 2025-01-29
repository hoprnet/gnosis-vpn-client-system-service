use gnosis_vpn_lib::{command, socket};

mod cli;

fn as_internal_cmd(cmd: &cli::Command) -> command::Command {
    match cmd {
        cli::Command::Status => command::Command::Status,
        cli::Command::EntryNode {
            endpoint,
            api_token,
            listen_host,
            hop,
            intermediate_id,
        } => command::Command::EntryNode {
            endpoint: endpoint.clone(),
            api_token: api_token.clone(),
            listen_host: listen_host.clone(),
            hop: *hop,
            intermediate_id: *intermediate_id,
        },
        cli::Command::ExitNode { peer_id } => command::Command::ExitNode { peer_id: *peer_id },
    }
}

fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let options = cli::cli().run();

    tracing::debug!(?options, "Options parsed");

    for cmd in options.commands.into_iter() {
        let cmd = as_internal_cmd(&cmd);
        let res = socket::process_cmd(&cmd);
        match res {
            Ok(socket::ReturnValue::WithResponse(s)) => tracing::info!("{} responded with: {}", cmd, s),
            Ok(_) => tracing::info!("{} executed successfully", cmd),
            Err(x) => tracing::warn!("{} failed with: {:?}", cmd, x),
        }
    }
}
