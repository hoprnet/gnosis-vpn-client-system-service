use futures::future::FutureExt;
use reqwest::header;
use reqwest::header::{HeaderMap, HeaderValue};
use std::thread;
use std::time::SystemTime;
use url::Url;

pub struct Core {
    status: Status,
    entry_node: Option<EntryNode>,
    client: Option<reqwest::Client>,
    entry_node_info: Option<EntryNodeInfo>,
}

enum Status {
    Starting, // => Idle
    Idle,
    OpeningSession { start_time: SystemTime },
}

struct EntryNode {
    endpoint: Url,
    api_token: String,
}

// TODO
struct EntryNodeInfo {
    addresses: serde_json::Value,
    peers: serde_json::Value,
}

impl Core {
    pub fn init() -> Core {
        Core {
            status: Status::Starting,
            entry_node: None,
            entry_node_info: None,
            client: Some(reqwest::Client::new()),
        }
    }

    pub fn started(&mut self) {
        self.status = Status::Idle;
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
                self.status = Status::OpeningSession {
                    start_time: SystemTime::now(),
                };
                self.query_entry_node_info();
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    pub fn to_string(&self) -> String {
        match self.status {
            Status::Starting => "starting".to_string(),
            Status::Idle => "idle".to_string(),
            Status::OpeningSession { start_time } => format!(
                "for {}ms: open session to {}",
                start_time.elapsed().unwrap().as_millis(),
                self.entry_node.as_ref().unwrap().endpoint
            ),
        }
    }

    async fn query_entry_node_info(&self) -> anyhow::Result<()> {
        if let (Some(entry_node), Some(client)) = (self.entry_node, self.client) {
            let (s_addr, r_addr) = crossbeam_channel::bounded(0);

            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
            let hv_token = HeaderValue::from_static(entry_node.api_token.as_str());
            hv_token.set_sensitive(true);
            headers.insert("x-auth-token", hv_token);

            let url_addresses = entry_node.endpoint.join("/api/v3/account/addresses")?;
            let url_peers = entry_node.endpoint.join("/api/v3/node/peers")?;

            let addresses = client
                .get(url_addresses)
                .headers(headers)
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?;

            s_addr.send(addresses);

            let (s_peers, r_peers) = crossbeam_channel::bounded(0);
            let peers = client
                .get(url_peers)
                .headers(headers)
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?;

            s_peers.send(peers);

            let addr = r_addr.recv();
            let peers = r_peers.recv();
        }
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
