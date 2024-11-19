use gnosis_vpn_lib::Command;
use reqwest::header;
use reqwest::header::{HeaderMap, HeaderValue};
use std::time::SystemTime;
use url::Url;
use std::thread;

// TODO
pub enum Event {
    GotAddresses { value: serde_json::Value },
    GotPeers { value: serde_json::Value },
}

pub struct Core {
    status: Status,
    entry_node: Option<EntryNode>,
    client: reqwest::Client,
    entry_node_addresses: Option<serde_json::Value>,
    entry_node_peers: Option<serde_json::Value>,
    sender: crossbeam_channel::Sender<Event>,
}

enum Status {
    Idle,
    OpeningSession { start_time: SystemTime },
}

struct EntryNode {
    endpoint: Url,
    api_token: String,
}

impl Core {
    pub fn init(sender: crossbeam_channel::Sender<Event>) -> Core {
        Core {
            status: Status::Idle,
            entry_node: None,
            entry_node_addresses: None,
            entry_node_peers: None,
            client: reqwest::Client::new(),
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
        }
    }

    pub fn handle_event(&mut self, event: Event) -> anyhow::Result<()> {
        match event {
            Event::GotAddresses { value } => {
                log::info!("got addresses: {}", value);
                self.entry_node_addresses = Some(value);
            }
            Event:: GotPeers { value } => {
                log::info!("got peers: {}", value);
                self.entry_node_peers = Some(value);
            }

        }
                Ok(())
    }

    pub fn status(&self) -> anyhow::Result<Option<String>> {
        Ok(Some(self.to_string()))
    }

    pub fn entry_node(
        &mut self,
        endpoint: Url,
        api_token: String,
    ) -> anyhow::Result<Option<String>> {
        self.entry_node = Some(EntryNode {
            endpoint,
            api_token,
        });

        match self.status {
            Status::Idle => {
                /*self.status = Status::OpeningSession {
                    start_time: SystemTime::now(),
                };*/
                self.query_entry_node_info()?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    pub fn to_string(&self) -> String {
        match self.status {
            Status::Idle => {
                let mut info = "idle".to_string();
                if let Some(entry_node) = &self.entry_node {
                    info = format!(
                        "{}: entry node: {}",
                        info,
                        entry_node.endpoint.as_str()
                    )
                }
                if let Some(entry_node_addresses) = &self.entry_node_addresses {
                    info = format!(
                        "{}: addresses: {}",
                        info,
                        entry_node_addresses
                    )
                }
                if let Some(entry_node_peers) = &self.entry_node_peers {
                    info = format!(
                        "{}: peers: {}",
                        info,
                        entry_node_peers
                    )
                }
                info
            },
            Status::OpeningSession { start_time } => format!(
                "for {}ms: open session to {}",
                start_time.elapsed().unwrap().as_millis(),
                self.entry_node.as_ref().unwrap().endpoint
            ),
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
            thread::spawn(|| async move {
                log::info!("querying addresses {}", url_addresses);
                let addresses = c1
                    .get(url_addresses)
                    .headers(h1)
                    .send()
                    .await
                    .unwrap()
                    .json::<serde_json::Value>()
                    .await
                    .unwrap();

                sender.send(Event::GotAddresses{ value: addresses}).unwrap();
            });

            let url_peers = entry_node.endpoint.join("/api/v3/node/peers")?;
            let sender = self.sender.clone();
            let c2 = self.client.clone();
            let h2 = headers.clone();
            thread::spawn(|| async move {
                log::info!("querying peers {}", url_peers);
                let peers = c2
                    .get(url_peers)
                    .headers(h2)
                    .send()
                    .await
                    .unwrap()
                    .json::<serde_json::Value>()
                    .await
                    .unwrap();

                sender.send(Event::GotPeers{ value: peers }).unwrap();
            });
        };
        Ok(())
    }

    /*
        fn open_session(&self) -> anyhow::Result<()> {
            let (sender, receiver) = crossbeam_channel::unbounded::<net::UnixStream>();
            let sender = sender.clone();

            let if Some(entry_node) =self.entry_node {
        thread::spawn(move || {

            let headers = headers::HeaderMap::new();
            headers.insert(headers::CONTENT_TYPE, HeaderValue::from_static("application/json"));
            headers.insert("x-auth-token", HeaderValue::from_static(entry_node.api_token));

            let body = serde_json::json!({
                "capabilities": [ "Segmentation"],
                "destination": "12D3KooWAjhroYkdRQMxp4ELS6uWpSTzLWd8vHx7292ztrkJ76gu",
                "path": { "Hops": 0 },
                "target": { "Plain": "wireguard.staging.hoprnet.link:51820"}
            });

            self.client.post(entry_node.endpoint)
                .headers(headers)
                .body(serde_json::to_string(body)))
                .send();
        })
            };

                let client =
            self.client
                .post(self.entry_node.as_ref().unwrap().endpoint)
                .header("Authorization", format

            for stream in listener.incoming() {
                _ = match stream {
                    Ok(stream) => sender
                        .send(stream)
                        .with_context(|| "failed to send stream to channel"),
                    Err(x) => {
                        log::error!("error waiting for incoming message: {:?}", x);
                        Err(anyhow!(x))
                    }
                };
            }
        });

        log::info!("started successfully in listening mode");
            crossbeam_channel::select! {
                recv(ctrl_c_events) -> _ => {
                    log::info!("shutting down");
                    break;
                }
                recv(receiver) -> stream => {
                    _ = match stream  {
                        Ok(s) => incoming_stream(state, s),
                        Err(x) => Err(anyhow!(x))

                    }
                },
            }
        }

    */
}
