use anyhow::Result;
use gnosis_vpn_lib::config::EntryNodeConfig;
use gnosis_vpn_lib::log_output;
use gnosis_vpn_lib::peer_id::PeerId;
use reqwest::blocking;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::thread;
use url::Url;

use crate::event::Event;
use crate::remote_data;

#[derive(Debug)]
pub struct EntryNode {
    input: Option<EntryNodeConfig>,
    addresses: Option<Addresses>,
}

#[derive(Debug)]
pub enum Path {
    Hop(u8),
    IntermediateId(PeerId),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Addresses {
    hopr: String,
    native: String,
}

pub fn schedule_retry_query_addresses(
    delay: std::time::Duration,
    sender: &crossbeam_channel::Sender<Event>,
) -> crossbeam_channel::Sender<()> {
    let sender = sender.clone();
    let (cancel_sender, cancel_receiver) = crossbeam_channel::bounded(1);
    thread::spawn(move || {
        crossbeam_channel::select! {
            recv(cancel_receiver) -> _ => {}
            default(delay) => {
                match sender.send(Event::FetchAddresses(remote_data::Event::Retry)) {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!(error = %e, "failed sending retry event");
                    }
                }
            }
        }
    });
    cancel_sender
}

pub fn schedule_retry_list_sessions(
    delay: std::time::Duration,
    sender: &crossbeam_channel::Sender<Event>,
) -> crossbeam_channel::Sender<()> {
    let sender = sender.clone();
    let (cancel_sender, cancel_receiver) = crossbeam_channel::bounded(1);
    thread::spawn(move || {
        crossbeam_channel::select! {
            recv(cancel_receiver) -> _ => {}
            default(delay) => {
            match sender.send(Event::FetchListSessions(remote_data::Event::Retry)) {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!(error = %e, "failed sending retry event");
                }
            }
            }
        }
    });
    cancel_sender
}

impl fmt::Display for EntryNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut print = HashMap::from([
            ("endpoint", self.endpoint.to_string()),
            ("api_token", "*****".to_string()),
        ]);
        print.insert("path", format!("{}", self.path));
        if let Some(listen_host) = &self.listen_host {
            print.insert("listen_host", listen_host.to_string());
        }
        // TODO avoid nesting json
        if let Some(addresses) = &self.addresses {
            print.insert("addresses", log_output::serialize(&addresses));
        }
        let val = log_output::serialize(&print);
        write!(f, "{}", val)
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Path::Hop(hop) => write!(f, "hop: {}", hop),
            Path::IntermediateId(peer_id) => write!(f, "intermediate_id: {}", peer_id),
        }
    }
}

impl EntryNode {
    pub fn new(endpoint: &Url, api_token: &str, listen_host: Option<&str>, path: Path) -> EntryNode {
        EntryNode {
            endpoint: endpoint.clone(),
            api_token: api_token.to_string(),
            addresses: None,
            listen_host: listen_host.map(|s| s.to_string()),
            path,
        }
    }

    pub fn query_addresses(&self, client: &blocking::Client, sender: &crossbeam_channel::Sender<Event>) -> Result<()> {
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

    pub fn list_sessions(&self, client: &blocking::Client, sender: &crossbeam_channel::Sender<Event>) -> Result<()> {
        let headers = remote_data::authentication_headers(self.api_token.as_str())?;
        let url = self.endpoint.join("/api/v3/session/udp")?;
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
                    Event::FetchListSessions(remote_data::Event::Response(json))
                }
                Ok((status, Ok(json))) => {
                    let e = remote_data::CustomError {
                        reqw_err: None,
                        status: Some(status),
                        value: Some(json),
                    };
                    Event::FetchListSessions(remote_data::Event::Error(e))
                }
                Ok((status, Err(e))) => {
                    let e = remote_data::CustomError {
                        reqw_err: Some(e),
                        status: Some(status),
                        value: None,
                    };
                    Event::FetchListSessions(remote_data::Event::Error(e))
                }
                Err(e) => {
                    let e = remote_data::CustomError {
                        reqw_err: Some(e),
                        status: None,
                        value: None,
                    };
                    Event::FetchListSessions(remote_data::Event::Error(e))
                }
            };
            sender.send(evt)
        });
        Ok(())
    }
}
