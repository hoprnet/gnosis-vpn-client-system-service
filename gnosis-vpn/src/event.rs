use crate::remote_data;
use serde::{Deserialize, Serialize};
use std::fmt;

pub enum Event {
    FetchAddresses(remote_data::Event<serde_json::Value>),
    // TODO
    GotPeers { value: serde_json::Value },
    // TODO
    GotSession { value: serde_json::Value },
    ListSesssions { resp: Vec<ListSessionsEntry> },
    CheckSession,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ListSessionsEntry {
    target: String,
    protocol: String,
    ip: String,
    port: u16,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Event::FetchAddresses(evt) => write!(f, "FetchAddresses: {}", evt),
            Event::GotPeers { value } => write!(f, "GotPeers: {}", value),
            Event::GotSession { value } => write!(f, "GotSession: {}", value),
            Event::ListSesssions { resp } => write!(f, "ListSesssions: {}", resp.len()),
            Event::CheckSession => write!(f, "CheckSession"),
        }
    }
}
