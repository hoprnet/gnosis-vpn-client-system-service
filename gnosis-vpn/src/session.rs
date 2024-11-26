use exponential_backoff::Backoff;
use reqwest::blocking;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::thread;
use std::time;
use url::Url;

use crate::entry_node::EntryNode;
use crate::event::Event;
use crate::exit_node::ExitNode;
use crate::remote_data;

#[derive(Serialize, Deserialize)]
pub struct Session {
    ip: String,
    port: u16,
    protocol: String,
    target: Url,
}

pub fn open_session_backoff() -> Backoff {
    let attempts = 3;
    let min = time::Duration::from_secs(1);
    let max = time::Duration::from_secs(5);
    Backoff::new(attempts, min, max)
}

pub fn open(
    client: &blocking::Client,
    sender: &crossbeam_channel::Sender<Event>,
    en: &EntryNode,
    xn: &ExitNode,
) -> anyhow::Result<()> {
    let headers = remote_data::authentication_headers(en.api_token.as_str())?;
    let url = en.endpoint.join("/api/v3/session/udp")?;
    let body = json!({
    "capabilities": ["Segmentation"],
    "destination": xn.peer_id,
    "path": {"Hops": 0 },
    "target": {"Plain": "wireguard.staging.hoprnet.link:51820"},
    "listenHost": format!("0.0.0.0:{}", en.session_port.unwrap_or(60006)),
    });
    let sender = sender.clone();
    let client = client.clone();
    thread::spawn(move || {
        let fetch_res = client
            .post(url)
            .json(&body)
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

        sender.send(evt);
    });
    Ok(())
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
                    tracing::warn!("sending delayed event failed: {:?}", e);
                }
            }
            }
        }
    });
    cancel_sender
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
                    tracing::warn!("sending delayed event failed: {:?}", e);
                }
            }
            }
        }
    });
    cancel_sender
}

impl Session {
    pub fn new(ip: String, port: u16, protocol: String, target: Url) -> Self {
        Self {
            ip,
            port,
            protocol,
            target,
        }
    }
    fn open(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn is_active(&self) -> anyhow::Result<bool> {
        Ok(false)
    }
}
