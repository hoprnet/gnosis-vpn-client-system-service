use gnosis_vpn_lib::command::Command;
use gnosis_vpn_lib::log_output;
use libp2p_identity::PeerId;
use reqwest::blocking;
use std::collections::HashMap;
use std::fmt;
use std::time;
use std::time::SystemTime;
use tracing::instrument;
use url::Url;

use crate::backoff;
use crate::backoff::FromIteratorToSeries;
use crate::entry_node;
use crate::entry_node::{EntryNode, Path};
use crate::event::Event; // Import the `entry_node` module // Import the `entry_node` module
use crate::exit_node::ExitNode;
use crate::remote_data;
use crate::remote_data::RemoteData;
use crate::session;
use crate::session::Session;

//
pub struct Core {
    status: Status,
    entry_node: Option<EntryNode>,
    exit_node: Option<ExitNode>,
    client: blocking::Client,
    fetch_data: FetchData,
    sender: crossbeam_channel::Sender<Event>,
    session: Option<Session>,
}

struct FetchData {
    addresses: RemoteData,
    open_session: RemoteData,
    list_sessions: RemoteData,
    close_session: RemoteData,
}

enum Status {
    Idle,
    OpeningSession {
        start_time: SystemTime,
    },
    MonitoringSession {
        start_time: SystemTime,
        cancel_sender: crossbeam_channel::Sender<()>,
    },
    ClosingSession {
        start_time: SystemTime,
    },
}

impl Core {
    pub fn init(sender: crossbeam_channel::Sender<Event>) -> Core {
        Core {
            status: Status::Idle,
            entry_node: None,
            exit_node: None,
            client: blocking::Client::new(),
            fetch_data: FetchData {
                addresses: RemoteData::NotAsked,
                open_session: RemoteData::NotAsked,
                list_sessions: RemoteData::NotAsked,
                close_session: RemoteData::NotAsked,
            },
            sender,
            session: None,
        }
    }

    #[instrument(level = tracing::Level::INFO, skip(self, cmd), ret(level = tracing::Level::DEBUG))]
    pub fn handle_cmd(&mut self, cmd: Command) -> anyhow::Result<Option<String>> {
        tracing::info!(%cmd, "Handling command");
        tracing::debug!(state_before = %self, "State cmd change");

        let res = match cmd {
            Command::Status => self.status(),
            Command::EntryNode {
                endpoint,
                api_token,
                listen_host,
                hop,
                intermediate_id,
            } => self.entry_node(endpoint, api_token, listen_host.clone(), hop, intermediate_id),
            Command::ExitNode { peer_id } => self.exit_node(peer_id),
        };

        tracing::debug!(state_after = %self, "State cmd change");

        res
    }

    #[instrument(level = tracing::Level::INFO, skip(self, event), ret(level = tracing::Level::DEBUG))]
    pub fn handle_event(&mut self, event: Event) -> anyhow::Result<()> {
        tracing::info!(%event, "Handling event");
        tracing::debug!(state_before = %self, "State evt change");

        let res = match event {
            Event::FetchAddresses(evt) => self.evt_fetch_addresses(evt),
            Event::FetchOpenSession(evt) => self.evt_fetch_open_session(evt),
            Event::FetchListSessions(evt) => self.evt_fetch_list_sessions(evt),
            Event::FetchCloseSession(evt) => self.evt_fetch_close_session(evt),
            Event::CheckSession => self.evt_check_session(),
        };

        tracing::debug!(state_after = %self, "State evt change");
        res
    }

    fn evt_fetch_addresses(&mut self, evt: remote_data::Event) -> anyhow::Result<()> {
        match evt {
            remote_data::Event::Response(value) => {
                self.fetch_data.addresses = RemoteData::Success;
                if let Some(en) = &mut self.entry_node {
                    let addresses = serde_json::from_value::<entry_node::Addresses>(value);
                    match addresses {
                        Ok(addr) => {
                            en.addresses = Some(addr);
                        }
                        Err(e) => {
                            tracing::error!("failed to parse addresses: {}", e);
                        }
                    }
                }
            }
            remote_data::Event::Error(err) => {
                match &self.fetch_data.addresses {
                    RemoteData::RetryFetching {
                        backoffs: old_backoffs, ..
                    } => {
                        let mut backoffs = old_backoffs.clone();
                        self.repeat_fetch_addresses(err, &mut backoffs)
                    }
                    RemoteData::Fetching { .. } => {
                        let mut backoffs = backoff::get_addresses().to_vec();
                        self.repeat_fetch_addresses(err, &mut backoffs);
                    }
                    _ => {
                        // should not happen
                        tracing::error!("unexpected event state");
                    }
                }
            }
            remote_data::Event::Retry => self.fetch_addresses()?,
        };
        Ok(())
    }

    fn evt_fetch_open_session(&mut self, evt: remote_data::Event) -> anyhow::Result<()> {
        match evt {
            remote_data::Event::Response(value) => {
                self.fetch_data.open_session = RemoteData::Success;
                let session = serde_json::from_value::<Session>(value);
                match session {
                    Ok(s) => {
                        self.session = Some(s);
                        let cancel_sender = session::schedule_check_session(time::Duration::from_secs(9), &self.sender);
                        self.status = Status::MonitoringSession {
                            start_time: SystemTime::now(),
                            cancel_sender,
                        };
                    }
                    Err(e) => {
                        tracing::error!("failed to parse session: {}", e);
                    }
                }
            }
            remote_data::Event::Error(err) => {
                match &self.fetch_data.open_session {
                    RemoteData::RetryFetching {
                        backoffs: old_backoffs, ..
                    } => {
                        let mut backoffs = old_backoffs.clone();
                        self.repeat_fetch_open_session(err, &mut backoffs)
                    }
                    RemoteData::Fetching { .. } => {
                        let mut backoffs = backoff::open_session().to_vec();
                        self.repeat_fetch_open_session(err, &mut backoffs);
                    }
                    _ => {
                        // should not happen
                        tracing::error!("unexpected event state");
                    }
                }
            }
            remote_data::Event::Retry => self.fetch_open_session()?,
        };
        Ok(())
    }

    fn evt_fetch_list_sessions(&mut self, evt: remote_data::Event) -> anyhow::Result<()> {
        match evt {
            remote_data::Event::Response(value) => {
                self.fetch_data.list_sessions = RemoteData::Success;
                let res_sessions = serde_json::from_value::<Vec<session::Session>>(value);
                match res_sessions {
                    Ok(sessions) => self.verify_session(&sessions),
                    Err(e) => {
                        tracing::error!("stopped monitoring - failed to parse sessions: {}", e);
                        self.status = Status::Idle;
                        Ok(())
                    }
                }
            }
            remote_data::Event::Error(err) => {
                match &self.fetch_data.list_sessions {
                    RemoteData::RetryFetching {
                        backoffs: old_backoffs, ..
                    } => {
                        let mut backoffs = old_backoffs.clone();
                        self.repeat_fetch_list_sessions(err, &mut backoffs)
                    }
                    RemoteData::Fetching { .. } => {
                        let mut backoffs = backoff::list_sessions().to_vec();
                        self.repeat_fetch_list_sessions(err, &mut backoffs)
                    }
                    _ => {
                        // should not happen
                        tracing::error!("unexpected event state");
                        Ok(())
                    }
                }
            }
            remote_data::Event::Retry => self.fetch_list_sessions(),
        }
    }

    fn evt_fetch_close_session(&mut self, evt: remote_data::Event) -> anyhow::Result<()> {
        match evt {
            remote_data::Event::Response(_) => {
                self.fetch_data.close_session = RemoteData::Success;
                self.status = Status::Idle;
                self.check_open_session()
            }
            remote_data::Event::Error(err) => {
                match &self.fetch_data.close_session {
                    RemoteData::RetryFetching {
                        backoffs: old_backoffs, ..
                    } => {
                        let mut backoffs = old_backoffs.clone();
                        self.repeat_fetch_close_session(err, &mut backoffs);
                        Ok(())
                    }
                    RemoteData::Fetching { .. } => {
                        let mut backoffs = backoff::close_session().to_vec();
                        self.repeat_fetch_close_session(err, &mut backoffs);
                        Ok(())
                    }
                    _ => {
                        // should not happen
                        tracing::error!("unexpected event state");
                        Ok(())
                    }
                }
            }
            remote_data::Event::Retry => self.fetch_close_session(),
        }
    }

    fn evt_check_session(&mut self) -> anyhow::Result<()> {
        match (&self.status, &self.fetch_data.list_sessions) {
            (_, RemoteData::Fetching { .. }) | (_, RemoteData::RetryFetching { .. }) => Ok(()),
            (Status::MonitoringSession { .. }, _) => {
                self.fetch_data.list_sessions = RemoteData::Fetching {
                    started_at: SystemTime::now(),
                };
                self.fetch_list_sessions()
            }
            _ => Ok(()),
        }
    }

    fn repeat_fetch_addresses(&mut self, error: remote_data::CustomError, backoffs: &mut Vec<time::Duration>) {
        if let Some(backoff) = backoffs.pop() {
            let cancel_sender = entry_node::schedule_retry_query_addresses(backoff, &self.sender);
            self.fetch_data.addresses = RemoteData::RetryFetching {
                error,
                cancel_sender,
                backoffs: backoffs.clone(),
            };
        } else {
            self.fetch_data.addresses = RemoteData::Failure { error };
        }
    }

    fn repeat_fetch_open_session(&mut self, error: remote_data::CustomError, backoffs: &mut Vec<time::Duration>) {
        if let Some(backoff) = backoffs.pop() {
            let cancel_sender = session::schedule_retry_open(backoff, &self.sender);
            self.fetch_data.open_session = RemoteData::RetryFetching {
                error,
                cancel_sender,
                backoffs: backoffs.clone(),
            };
        } else {
            self.fetch_data.open_session = RemoteData::Failure { error };
            self.status = Status::Idle;
        }
    }

    fn repeat_fetch_list_sessions(
        &mut self,
        error: remote_data::CustomError,
        backoffs: &mut Vec<time::Duration>,
    ) -> anyhow::Result<()> {
        if let Some(backoff) = backoffs.pop() {
            let cancel_sender = entry_node::schedule_retry_list_sessions(backoff, &self.sender);
            self.fetch_data.list_sessions = RemoteData::RetryFetching {
                error,
                cancel_sender,
                backoffs: backoffs.clone(),
            };
            Ok(())
        } else {
            self.fetch_data.list_sessions = RemoteData::Failure { error };
            if let Status::MonitoringSession { .. } = self.status {
                self.check_close_session()
            } else {
                tracing::warn!("failed list session call while not monitoring session");
                Ok(())
            }
        }
    }

    fn repeat_fetch_close_session(&mut self, error: remote_data::CustomError, backoffs: &mut Vec<time::Duration>) {
        if let Some(backoff) = backoffs.pop() {
            let cancel_sender = session::schedule_retry_close(backoff, &self.sender);
            self.fetch_data.close_session = RemoteData::RetryFetching {
                error,
                cancel_sender,
                backoffs: backoffs.clone(),
            };
        } else {
            self.fetch_data.close_session = RemoteData::Failure { error };
            if let Status::ClosingSession { .. } = self.status {
                self.status = Status::Idle;
            } else {
                tracing::warn!("failed close session call while not closing session");
            }
        }
    }

    fn status(&self) -> anyhow::Result<Option<String>> {
        Ok(Some(self.to_string()))
    }

    fn entry_node(
        &mut self,
        endpoint: Url,
        api_token: String,
        listen_port: Option<String>,
        hop: Option<u8>,
        intermediate_id: Option<PeerId>,
    ) -> anyhow::Result<Option<String>> {
        self.check_close_session()?;

        // TODO move this to library and enhance CLI to only allow one option
        // hop has precedence over intermediate_id
        let path = match (hop, intermediate_id) {
            (Some(h), _) => Path::Hop(h),
            (_, Some(id)) => Path::IntermediateId(id),
            _ => Path::Hop(1),
        };
        self.entry_node = Some(EntryNode::new(endpoint, api_token, listen_port, path));
        self.fetch_data.addresses = RemoteData::Fetching {
            started_at: SystemTime::now(),
        };
        self.fetch_addresses()?;
        self.check_open_session()?;
        Ok(None)
    }

    fn exit_node(&mut self, peer_id: PeerId) -> anyhow::Result<Option<String>> {
        self.check_close_session()?;
        self.exit_node = Some(ExitNode { peer_id });
        self.check_open_session()?;
        Ok(None)
    }

    fn check_open_session(&mut self) -> anyhow::Result<()> {
        match (&self.status, &self.entry_node, &self.exit_node) {
            (Status::Idle, Some(_), Some(_)) => {
                self.status = Status::OpeningSession {
                    start_time: SystemTime::now(),
                };
                self.fetch_data.open_session = RemoteData::Fetching {
                    started_at: SystemTime::now(),
                };
                self.fetch_open_session()
            }
            _ => Ok(()),
        }
    }

    fn check_close_session(&mut self) -> anyhow::Result<()> {
        self.cancel_fetch_addresses();
        self.cancel_fetch_open_session();
        self.cancel_fetch_list_sessions();
        self.cancel_fetch_close_session();
        self.cancel_session_monitoring();
        match &self.status {
            Status::MonitoringSession { .. } => {
                self.status = Status::ClosingSession {
                    start_time: SystemTime::now(),
                };
                self.fetch_data.close_session = RemoteData::Fetching {
                    started_at: SystemTime::now(),
                };
                self.fetch_close_session()
            }
            _ => Ok(()),
        }
    }

    fn fetch_addresses(&mut self) -> anyhow::Result<()> {
        match &self.entry_node {
            Some(en) => en.query_addresses(&self.client, &self.sender),
            _ => Ok(()),
        }
    }

    fn fetch_open_session(&mut self) -> anyhow::Result<()> {
        match (&self.entry_node, &self.exit_node) {
            (Some(en), Some(xn)) => session::open(&self.client, &self.sender, en, xn),
            _ => Ok(()),
        }
    }

    fn fetch_list_sessions(&mut self) -> anyhow::Result<()> {
        match &self.entry_node {
            Some(en) => en.list_sessions(&self.client, &self.sender),
            _ => Ok(()),
        }
    }

    fn fetch_close_session(&mut self) -> anyhow::Result<()> {
        match (&self.entry_node, &self.session) {
            (Some(en), Some(sess)) => sess.close(&self.client, &self.sender, en),
            _ => Ok(()),
        }
    }

    fn verify_session(&mut self, sessions: &[session::Session]) -> anyhow::Result<()> {
        match (&self.session, &self.status) {
            (Some(sess), Status::MonitoringSession { start_time, .. }) => {
                if sess.verify_open(sessions) {
                    tracing::info!("{} verified open since {}", sess, log_output::elapsed(start_time));
                    let cancel_sender = session::schedule_check_session(time::Duration::from_secs(9), &self.sender);
                    self.status = Status::MonitoringSession {
                        start_time: *start_time,
                        cancel_sender,
                    };
                    Ok(())
                } else {
                    tracing::info!("session no longer open");
                    self.status = Status::Idle;
                    self.check_open_session()
                }
            }
            (Some(sess), _) => {
                tracing::warn!("skip verifying session {} - no longer monitoring", sess);
                Ok(())
            }
            (None, status) => {
                tracing::warn!("skip verifiying session - no session to verify in status {}", status);
                Ok(())
            }
        }
    }

    fn cancel_fetch_addresses(&self) {
        if let RemoteData::RetryFetching { cancel_sender, .. } = &self.fetch_data.addresses {
            let res = cancel_sender.send(());
            match res {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("sending cancel event failed: {}", e);
                }
            }
        }
    }

    fn cancel_fetch_open_session(&self) {
        if let RemoteData::RetryFetching { cancel_sender, .. } = &self.fetch_data.open_session {
            let res = cancel_sender.send(());
            match res {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("sending cancel event failed: {}", e);
                }
            }
        }
    }

    fn cancel_fetch_list_sessions(&self) {
        if let RemoteData::RetryFetching { cancel_sender, .. } = &self.fetch_data.list_sessions {
            let res = cancel_sender.send(());
            match res {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("sending cancel event failed: {}", e);
                }
            }
        }
    }

    fn cancel_fetch_close_session(&self) {
        if let RemoteData::RetryFetching { cancel_sender, .. } = &self.fetch_data.close_session {
            let res = cancel_sender.send(());
            match res {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("sending cancel event failed: {}", e);
                }
            }
        }
    }

    fn cancel_session_monitoring(&self) {
        if let Status::MonitoringSession { cancel_sender, .. } = &self.status {
            let res = cancel_sender.send(());
            match res {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("sending cancel event failed: {}", e);
                }
            }
        }
    }
}

impl fmt::Display for ExitNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let peer = self.peer_id.to_base58();
        let print = HashMap::from([("peer_id", peer.as_str())]);
        let val = log_output::serialize(&print);
        write!(f, "{}", val)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let val = match self {
            Status::Idle => "idle",
            Status::OpeningSession { start_time } => {
                &format!("opening session since {}", log_output::elapsed(start_time)).to_string()
            }
            Status::MonitoringSession { start_time, .. } => {
                &format!("monitoring session since {}", log_output::elapsed(start_time)).to_string()
            }
            Status::ClosingSession { start_time } => {
                &format!("closing session since {}", log_output::elapsed(start_time)).to_string()
            }
        };
        write!(f, "{}", val)
    }
}

impl fmt::Display for Core {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut print = HashMap::from([("status", self.status.to_string())]);
        if let Some(en) = &self.entry_node {
            print.insert("entry_node", en.to_string());
        }
        if let Some(xn) = &self.exit_node {
            print.insert("exit_node", xn.to_string());
        }
        let val = log_output::serialize(&print);
        write!(f, "{}", val)
    }
}
