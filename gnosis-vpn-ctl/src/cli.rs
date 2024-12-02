use bpaf::Bpaf;
use libp2p_identity::PeerId;
use url::Url;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Cli {
    #[bpaf(external(command), many)]
    pub commands: Vec<Command>,
}

#[derive(Debug, Clone, Bpaf)]
pub enum Command {
    /// Specifies the entry node
    #[bpaf(command, adjacent)]
    EntryNode {
        #[bpaf(short, long)]
        endpoint: Url,
        #[bpaf(short, long)]
        api_token: String,
        #[bpaf(
            short,
            long,
            help(
                "Listen host can be provided like this: \"<host>:<port>\" or any combination thereof, e.g.: \":port\"."
            )
        )]
        listen_host: Option<String>,
        #[bpaf(short, long, guard(maxhop, "must be less or equal to 3"))]
        hop: Option<u8>,
    },
    /// Specifies the exit node
    #[bpaf(command, adjacent)]
    ExitNode {
        #[bpaf(short, long)]
        peer_id: PeerId,
    },
    /// Displays the current status
    #[bpaf(command, adjacent)]
    Status,
}

fn maxhop(hop: &Option<u8>) -> bool {
    match hop {
        Some(h) => *h <= 3,
        None => false,
    }
}
