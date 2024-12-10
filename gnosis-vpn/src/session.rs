use reqwest::blocking;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cmp;
use std::fmt;
use std::thread;
use url::Url;

use crate::core::error::Error as CoreError;
use crate::entry_node::{EntryNode, Path};
use crate::event::Event;
use crate::exit_node::ExitNode;
use crate::remote_data;

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    // listen host
    ip: String,
    port: u16,
    protocol: String,
    target: Url,
}

#[tracing::instrument(skip(client, sender), level = tracing::Level::DEBUG)]
pub fn open(
    client: &blocking::Client,
    sender: &crossbeam_channel::Sender<Event>,
    en: &EntryNode,
    xn: &ExitNode,
) -> Result<(), CoreError> {
    let headers = remote_data::authentication_headers(en.api_token.as_str())?;
    let url = match en.endpoint.join("/api/v3/session/udp") {
        Ok(url) => url,
        Err(e) => {
            return Err(CoreError::Url(e));
        }
    };

    let mut json = serde_json::Map::new();
    json.insert("capabilities".to_string(), json!(["Segmentation"]));
    json.insert("destination".to_string(), json!(xn.peer_id.to_base58()));
    json.insert(
        "target".to_string(),
        json!({"Plain": "wireguard.staging.hoprnet.link:51820"}),
    );
    match en.path {
        Path::Hop(hop) => {
            json.insert("path".to_string(), json!({"Hops": hop}));
        }
        Path::IntermediateId(id) => {
            json.insert("path".to_string(), json!({ "IntermediatePath": [id.to_base58()]}));
        }
    }
    if let Some(lh) = &en.listen_host {
        json.insert("listenHost".to_string(), json!(lh));
    };

    let sender = sender.clone();
    let client = client.clone();
    thread::spawn(move || {
        tracing::debug!(
            "post open session [headers: {:?}, body: {:?}, url: {:?}",
            headers,
            json,
            url
        );

        let fetch_res = client
            .post(url)
            .json(&json)
            .timeout(std::time::Duration::from_secs(30))
            .headers(headers)
            .send()
            .map(|res| (res.status(), res.json::<serde_json::Value>()));

        let evt = match fetch_res {
            Ok((status, Ok(json))) if status.is_success() => {
                Event::FetchOpenSession(remote_data::Event::Response(json))
            }
            Ok((status, Ok(json))) => {
                let e = remote_data::CustomError {
                    reqw_err: None,
                    status: Some(status),
                    value: Some(json),
                };
                Event::FetchOpenSession(remote_data::Event::Error(e))
            }
            Ok((status, Err(e))) => {
                let e = remote_data::CustomError {
                    reqw_err: Some(e),
                    status: Some(status),
                    value: None,
                };
                Event::FetchOpenSession(remote_data::Event::Error(e))
            }
            Err(e) => {
                let e = remote_data::CustomError {
                    reqw_err: Some(e),
                    status: None,
                    value: None,
                };
                Event::FetchOpenSession(remote_data::Event::Error(e))
            }
        };

        sender.send(evt)
    });
    Ok(())
}

pub fn schedule_check_session(
    delay: std::time::Duration,
    sender: &crossbeam_channel::Sender<Event>,
) -> crossbeam_channel::Sender<()> {
    let sender = sender.clone();
    let (cancel_sender, cancel_receiver) = crossbeam_channel::bounded(1);
    thread::spawn(move || {
        crossbeam_channel::select! {
            recv(cancel_receiver) -> _ => {}
            default(delay) => {
                let res = sender.send(Event::CheckSession);
            match res {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("sending delayed event failed: {}", e);
                }
            }
            }
        }
    });
    cancel_sender
}

pub fn schedule_retry_open(
    delay: std::time::Duration,
    sender: &crossbeam_channel::Sender<Event>,
) -> crossbeam_channel::Sender<()> {
    let sender = sender.clone();
    let (cancel_sender, cancel_receiver) = crossbeam_channel::bounded(1);
    thread::spawn(move || {
        crossbeam_channel::select! {
            recv(cancel_receiver) -> _ => {}
            default(delay) => {
            let res = sender.send(Event::FetchOpenSession(remote_data::Event::Retry));
            match res {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("sending delayed event failed: {}", e);
                }
            }
            }
        }
    });
    cancel_sender
}

pub fn schedule_retry_close(
    delay: std::time::Duration,
    sender: &crossbeam_channel::Sender<Event>,
) -> crossbeam_channel::Sender<()> {
    let sender = sender.clone();
    let (cancel_sender, cancel_receiver) = crossbeam_channel::bounded(1);
    thread::spawn(move || {
        crossbeam_channel::select! {
            recv(cancel_receiver) -> _ => {}
            default(delay) => {
            let res = sender.send(Event::FetchCloseSession(remote_data::Event::Retry));
            match res {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("sending delayed event failed: {}", e);
                }
            }
            }
        }
    });
    cancel_sender
}

impl Session {
    pub fn verify_open(&self, sessions: &[Session]) -> bool {
        sessions.iter().any(|entry| entry == self)
    }

    pub fn close(
        &self,
        client: &blocking::Client,
        sender: &crossbeam_channel::Sender<Event>,
        entry_node: &EntryNode,
    ) -> Result<(), CoreError> {
        let headers = remote_data::authentication_headers(entry_node.api_token.as_str())?;
        let url = match entry_node.endpoint.join("/api/v3/session/udp") {
            Ok(url) => url,
            Err(e) => {
                return Err(CoreError::Url(e));
            }
        };

        let mut json = serde_json::Map::new();
        json.insert("listeningIp".to_string(), json!(self.ip));
        json.insert("port".to_string(), json!(self.port));

        let sender = sender.clone();
        let client = client.clone();

        thread::spawn(move || {
            tracing::debug!(
                "delete session [headers: {:?}, body: {:?}, url: {:?}",
                headers,
                json,
                url
            );

            let fetch_res = client
                .delete(url)
                .json(&json)
                .timeout(std::time::Duration::from_secs(30))
                .headers(headers)
                .send()
                .map(|res| (res.status(), res.json::<serde_json::Value>()));

            let evt = match fetch_res {
                Ok((status, Ok(json))) if status.is_success() => {
                    Event::FetchCloseSession(remote_data::Event::Response(json))
                }
                Ok((status, Ok(json))) => {
                    let e = remote_data::CustomError {
                        reqw_err: None,
                        status: Some(status),
                        value: Some(json),
                    };
                    Event::FetchCloseSession(remote_data::Event::Error(e))
                }
                // TODO hanlde empty expected response better
                Ok((status, Err(_))) if status.is_success() => {
                    Event::FetchCloseSession(remote_data::Event::Response(serde_json::Value::Null))
                }
                Ok((status, Err(e))) => {
                    let e = remote_data::CustomError {
                        reqw_err: Some(e),
                        status: Some(status),
                        value: None,
                    };
                    Event::FetchCloseSession(remote_data::Event::Error(e))
                }
                Err(e) => {
                    let e = remote_data::CustomError {
                        reqw_err: Some(e),
                        status: None,
                        value: None,
                    };
                    Event::FetchCloseSession(remote_data::Event::Error(e))
                }
            };

            sender.send(evt)
        });
        Ok(())
    }
}

impl cmp::PartialEq for Session {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip
            && self.port == other.port
            && self.protocol == other.protocol
            && self.target.as_str().eq_ignore_ascii_case(other.target.as_str())
    }
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Session[{}:{} {} {}]",
            self.ip, self.port, self.protocol, self.target
        )
    }
}
