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
            ),
            guard(
                valid_listen_host,
                r#"must be in the form of ":<port>", "<host>" or "<host>:<port>""#
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

fn valid_listen_host(listen_host: &Option<String>) -> bool {
    match listen_host {
        Some(lh) => {
            let parts: Vec<&str> = lh.split(':').collect();
            match parts.len() {
                1 => url::Host::parse(parts[0]).is_ok(),
                2 => {
                    let host_ok = if parts[0].is_empty() {
                        true
                    } else {
                        url::Host::parse(parts[0]).is_ok()
                    };
                    let port_ok = if let Ok(port) = parts[1].parse::<u16>() {
                        u16::MIN <= port && port <= u16::MAX
                    } else {
                        false
                    };
                    host_ok && port_ok
                }
                _ => false,
            }
        }
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::valid_listen_host;
    #[test]
    fn test_valid_listen_host() {
        assert!(valid_listen_host(&Some("0.0.0.0:60006".to_string())));
        assert!(valid_listen_host(&Some(":60006".to_string())));
        assert!(valid_listen_host(&Some("0.0.0.0".to_string())));
        assert!(valid_listen_host(&Some("localhost:0".to_string())));
        assert!(!valid_listen_host(&Some("".to_string())));
        assert!(!valid_listen_host(&Some("localhost:".to_string())));
        assert!(!valid_listen_host(&Some("localhost:abc".to_string())));
        assert!(!valid_listen_host(&Some("localhost:65536".to_string())));
        assert!(!valid_listen_host(&Some("localhost:-1".to_string())));
    }
}
