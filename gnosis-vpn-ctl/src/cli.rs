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
        #[bpaf(short, long)]
        session_port: Option<u16>,
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
