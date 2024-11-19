use gnosis_vpn_lib::Command;
use reqwest::blocking;
use reqwest::header;
use reqwest::header::{HeaderMap, HeaderValue};
use std::fmt;
use std::thread;
use std::time::SystemTime;
use url::Url;

// TODO
pub enum Event {
    GotAddresses { value: serde_json::Value },
    GotPeers { value: serde_json::Value },
    GotSession { value: serde_json::Value },
    ListSesssions { value: serde_json::Value },
    CheckSession,
}

pub struct Core {
    status: Status,
    entry_node: Option<EntryNode>,
    exit_node: Option<ExitNode>,
    client: blocking::Client,
    entry_node_addresses: Option<serde_json::Value>,
    entry_node_peers: Option<serde_json::Value>,
    entry_node_session: Option<serde_json::Value>,
    sender: crossbeam_channel::Sender<Event>,
}

enum Status {
    Idle,
    OpeningSession { start_time: SystemTime },
    MonitoringSession { start_time: SystemTime, port: u16 },
    ListingSessions { start_time: SystemTime },
}

struct EntryNode {
    endpoint: Url,
    api_token: String,
}

struct ExitNode {
    peer_id: String,
}

impl Core {
    pub fn init(sender: crossbeam_channel::Sender<Event>) -> Core {
        Core {
            status: Status::Idle,
            entry_node: None,
            exit_node: None,
            entry_node_addresses: None,
            entry_node_peers: None,
            entry_node_session: None,
            client: blocking::Client::new(),
            sender,
        }
    }

    pub fn handle_cmd(&mut self, cmd: gnosis_vpn_lib::Command) -> anyhow::Result<Option<String>> {
        log::info!("handling command: {}", cmd);
        match cmd {
            Command::Status => self.status(),
            Command::EntryNode {
                endpoint,
                api_token,
            } => self.entry_node(endpoint, api_token),
            Command::ExitNode { peer_id } => self.exit_node(peer_id),
        }
    }

    pub fn handle_event(&mut self, event: Event) -> anyhow::Result<()> {
        log::info!("handling event: {}", event);
        match event {
            Event::GotAddresses { value } => {
                self.entry_node_addresses = Some(value);
            }
            Event::GotPeers { value } => {
                self.entry_node_peers = Some(value);
            }

            Event::GotSession { value } => {
                self.entry_node_session = Some(value);
                self.status = Status::MonitoringSession {
                    start_time: SystemTime::now(),
                    port: 60006,
                };
                self.check_list_sessions()?;
            }
            Event::CheckSession => {
                self.check_list_sessions()?;
            }
            Event::ListSesssions { value } => {
                log::info!("todo");
            }
        }
        Ok(())
    }

    pub fn to_string(&self) -> String {
        match self.status {
            Status::Idle => {
                let mut info = "idle".to_string();
                if let Some(entry_node) = &self.entry_node {
                    info = format!("{} | entry node: {}", info, entry_node.endpoint.as_str())
                }
                if let Some(entry_node_addresses) = &self.entry_node_addresses {
                    info = format!("{} | addresses: {}", info, entry_node_addresses)
                }
                if let Some(entry_node_peers) = &self.entry_node_peers {
                    info = format!("{} | peers: {}", info, entry_node_peers)
                }
                if let Some(exit_node) = &self.exit_node {
                    info = format!("{} | exit_node: {}", info, exit_node.peer_id.as_str())
                }
                info
            }
            Status::OpeningSession { start_time } => format!(
                "for {}ms: open session to {}",
                start_time.elapsed().unwrap().as_millis(),
                self.entry_node.as_ref().unwrap().endpoint
            ),
            Status::MonitoringSession { start_time, port } => format!(
                "for {}ms: monitoring session on port {}",
                start_time.elapsed().unwrap().as_millis(),
                port
            ),
            Status::ListingSessions { start_time } => format!(
                "for {}ms: listing sessions",
                start_time.elapsed().unwrap().as_millis()
            ),
        }
    }

    fn status(&self) -> anyhow::Result<Option<String>> {
        Ok(Some(self.to_string()))
    }

    fn entry_node(&mut self, endpoint: Url, api_token: String) -> anyhow::Result<Option<String>> {
        self.entry_node = Some(EntryNode {
            endpoint,
            api_token,
        });
        self.query_entry_node_info()?;
        self.check_open_session()?;
        Ok(None)
    }

    fn exit_node(&mut self, peer_id: String) -> anyhow::Result<Option<String>> {
        self.exit_node = Some(ExitNode { peer_id });
        self.check_open_session()?;
        Ok(None)
    }

    fn check_open_session(&mut self) -> anyhow::Result<()> {
        match (&self.status, &self.entry_node, &self.exit_node) {
            (Status::Idle, Some(entry_node), Some(exit_node)) => {
                self.status = Status::OpeningSession {
                    start_time: SystemTime::now(),
                };
                self.open_session(entry_node, exit_node)
            }
            _ => Ok(()),
        }
    }

    fn check_list_sessions(&mut self) -> anyhow::Result<()> {
        match (&self.status, &self.entry_node) {
            (
                Status::MonitoringSession {
                    start_time,
                    port: _,
                },
                Some(entry_node),
            ) => {
                if start_time.elapsed().unwrap().as_secs() > 3 {
                    self.status = Status::ListingSessions {
                        start_time: SystemTime::now(),
                    };
                    self.list_sessions(entry_node)
                } else {
                    let sender = self.sender.clone();
                    thread::spawn(move || {
                        thread::sleep(std::time::Duration::from_millis(333));
                        sender.send(Event::CheckSession).unwrap();
                    });
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    fn query_entry_node_info(&mut self) -> anyhow::Result<()> {
        if let Some(entry_node) = &self.entry_node {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
            let mut hv_token = HeaderValue::from_str(entry_node.api_token.as_str())?;
            hv_token.set_sensitive(true);
            headers.insert("x-auth-token", hv_token);

            let url_addresses = entry_node.endpoint.join("/api/v3/account/addresses")?;
            let sender = self.sender.clone();
            let c1 = self.client.clone();
            let h1 = headers.clone();
            thread::spawn(move || {
                let addresses = c1
                    .get(url_addresses)
                    .headers(h1)
                    .send()
                    .unwrap()
                    .json::<serde_json::Value>()
                    .unwrap();

                sender
                    .send(Event::GotAddresses { value: addresses })
                    .unwrap();
            });

            let url_peers = entry_node.endpoint.join("/api/v3/node/peers")?;
            let sender = self.sender.clone();
            let c2 = self.client.clone();
            let h2 = headers.clone();
            thread::spawn(move || {
                let peers = c2
                    .get(url_peers)
                    .headers(h2)
                    .send()
                    .unwrap()
                    .json::<serde_json::Value>()
                    .unwrap();

                sender.send(Event::GotPeers { value: peers }).unwrap();
            });
        };
        Ok(())
    }

    fn open_session(&self, entry_node: &EntryNode, exit_node: &ExitNode) -> anyhow::Result<()> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        let mut hv_token = HeaderValue::from_str(entry_node.api_token.as_str())?;
        hv_token.set_sensitive(true);
        headers.insert("x-auth-token", hv_token);

        let body = serde_json::json!({
            "capabilities": ["Segmentation"],
            "destination": exit_node.peer_id,
            "path": {"Hops": 0},
            "target": {"Plain": "wireguard.staging.hoprnet.link:51820"},
            "listenHost": "127.0.0.1:60006"
        });

        let url = entry_node.endpoint.join("/api/v3/session/udp")?;
        let sender = self.sender.clone();
        let c = self.client.clone();
        let h = headers.clone();
        thread::spawn(move || {
            let session = c
                .post(url)
                .headers(h)
                .json(&body)
                .send()
                .unwrap()
                .json::<serde_json::Value>()
                .unwrap();

            sender.send(Event::GotSession { value: session }).unwrap();
        });

        Ok(())
    }

    fn list_sessions(&self, entry_node: &EntryNode) -> anyhow::Result<()> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        let mut hv_token = HeaderValue::from_str(entry_node.api_token.as_str())?;
        hv_token.set_sensitive(true);
        headers.insert("x-auth-token", hv_token);

        let url = entry_node.endpoint.join("/api/v3/session/udp")?;
        let sender = self.sender.clone();
        let c = self.client.clone();
        let h = headers.clone();
        thread::spawn(move || {
            let sessions = c
                .get(url)
                .headers(h)
                .send()
                .unwrap()
                .json::<serde_json::Value>()
                .unwrap();

            sender
                .send(Event::ListSesssions { value: sessions })
                .unwrap();
        });
        Ok(())
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Event::GotAddresses { value } => write!(f, "GotAddresses: {}", value),
            Event::GotPeers { value } => write!(f, "GotPeers: {}", value),
            Event::GotSession { value } => write!(f, "GotSession: {}", value),
            Event::ListSesssions { value } => write!(f, "ListSesssions: {}", value),
            Event::CheckSession => write!(f, "CheckSession"),
        }
    }
}
