use crate::event::Event;
use crate::remote_data;
use exponential_backoff::Backoff;
use reqwest::blocking;
use std::collections::HashMap;
use std::fmt;
use std::thread;
use std::time;
use url::Url;

pub struct EntryNode {
    endpoint: Url,
    api_token: String,
}

pub fn addressses_backoff() -> Backoff {
    let attempts = 10;
    let min = time::Duration::from_secs(1);
    let max = time::Duration::from_secs(30);
    Backoff::new(attempts, min, max)
}

pub fn schedule_retry(
    delay: std::time::Duration,
    sender: &crossbeam_channel::Sender<Event>,
) -> crossbeam_channel::Sender<()> {
    let sender = sender.clone();
    let (cancel_sender, cancel_receiver) = crossbeam_channel::bounded(1);
    crossbeam_channel::select! {
        recv(cancel_receiver) -> _ => {}
        default(delay) => {
        sender.send(Event::FetchAddresses(remote_data::Event::Retry));

        }
    }
    cancel_sender
}

impl fmt::Display for EntryNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let print = HashMap::from([("endpoint", self.endpoint.as_str()), ("api_token", "*****")]);
        let val = serde_json::to_string(&print).unwrap();
        write!(f, "{}", val)
    }
}

impl EntryNode {
    pub fn new(endpoint: Url, api_token: String) -> EntryNode {
        EntryNode {
            endpoint,
            api_token,
        }
    }

    pub fn query_addresses(
        &self,
        client: &blocking::Client,
        sender: &crossbeam_channel::Sender<Event>,
    ) -> anyhow::Result<()> {
        let headers = remote_data::authentication_headers(self.api_token.as_str())?;
        let url = self.endpoint.join("/api/v3/account/addresses")?;

        let sender = sender.clone();
        let client = client.clone();
        thread::spawn(move || {
            let fetch_res = client
                .get(url)
                .timeout(std::time::Duration::from_secs(30))
                .headers(headers)
                .send();

            tracing::info!("fetch_res: {:?}", fetch_res);

            let foo = fetch_res.is_ok();
            tracing::info!("foo: {:?}", foo);

            match fetch_res {
                Ok(res) => {
                    let json_res = res.json::<serde_json::Value>();
                    tracing::info!("json_res: {:?}", json_res);

                    match json_res {
                        Ok(json) => {
                            tracing::info!("json: {:?}", json);
                            let evt = Event::FetchAddresses(remote_data::Event::Response(json));
                            sender.send(evt)
                        }
                        Err(e) => {
                            tracing::info!("json err: {:?}", e);
                            let evt = Event::FetchAddresses(remote_data::Event::Error(e));
                            sender.send(evt)
                        }
                    }
                }
                Err(e) => {
                    tracing::info!("fetch err: {:?}", e);
                    let evt = Event::FetchAddresses(remote_data::Event::Error(e));
                    sender.send(evt)
                }
            }
        });
        Ok(())
    }
}
