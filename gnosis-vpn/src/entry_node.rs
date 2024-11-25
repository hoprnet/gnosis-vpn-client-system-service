use crate::event::Event;
use crate::remote_data;
use exponential_backoff::Backoff;
use reqwest::blocking;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::thread;
use std::time;
use url::Url;

pub struct EntryNode {
    pub endpoint: Url,
    pub api_token: String,
    pub addresses: Option<Addresses>,
}

#[derive(Serialize, Deserialize)]
pub struct Addresses {
    hopr: String,
    native: String,
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
    thread::spawn(move || {
        crossbeam_channel::select! {
            recv(cancel_receiver) -> _ => {}
            default(delay) => {
            let res = sender.send(Event::FetchAddresses(remote_data::Event::Retry));
            match res {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("sending retry event failed: {:?}", e);
                }
            }
            }
        }
    });
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
            addresses: None,
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
                .send()
                .map(|res| (res.status(), res.json::<serde_json::Value>()));

            let evt = match fetch_res {
                Ok((status, Ok(json))) if status.is_success() => {
                    Event::FetchAddresses(remote_data::Event::Response(json))
                }
                Ok((status, Ok(json))) => {
                    let e = remote_data::CustomError {
                        reqw_err: None,
                        status: Some(status),
                        value: Some(json),
                    };
                    Event::FetchAddresses(remote_data::Event::Error(e))
                }
                Ok((status, Err(e))) => {
                    let e = remote_data::CustomError {
                        reqw_err: Some(e),
                        status: Some(status),
                        value: None,
                    };
                    Event::FetchAddresses(remote_data::Event::Error(e))
                }
                Err(e) => {
                    let e = remote_data::CustomError {
                        reqw_err: Some(e),
                        status: None,
                        value: None,
                    };
                    Event::FetchAddresses(remote_data::Event::Error(e))
                }
            };
            sender.send(evt)
        });
        Ok(())
    }
}
