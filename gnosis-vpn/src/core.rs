use gnosis_vpn_lib::Command;
use reqwest::blocking;
use std::collections::HashMap;
use std::fmt;
use std::time;
use std::time::SystemTime;
use url::Url;

use crate::entry_node;
use crate::entry_node::EntryNode;
use crate::event;
use crate::event::Event; // Import the `entry_node` module // Import the `entry_node` module
use crate::exit_node::ExitNode;
use crate::remote_data;
use crate::remote_data::RemoteData;
use crate::session;
use crate::session::Session;

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
    ListingSessions {
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
            },
            sender,
            session: None,
        }
    }

    pub fn handle_cmd(&mut self, cmd: gnosis_vpn_lib::Command) -> anyhow::Result<Option<String>> {
        tracing::trace!("HANDLE CMD [state before]: {}", self);
        tracing::debug!("HANDLE CMD [cmd]: {}", cmd);

        let res = match cmd {
            Command::Status => self.status(),
            Command::EntryNode {
                endpoint,
                api_token,
                session_port,
            } => self.entry_node(endpoint, api_token, session_port),
            Command::ExitNode { peer_id } => self.exit_node(peer_id),
        };

        tracing::trace!("HANDLE CMD [state after]: {}", self);
        res
    }

    pub fn handle_event(&mut self, event: Event) -> anyhow::Result<()> {
        tracing::trace!("HANDLE EVENT [state before]: {}", self);
        tracing::debug!("HANDLE EVENT [event]: {}", event);

        let res = match event {
            Event::FetchAddresses(evt) => self.evt_fetch_addresses(evt),
            Event::FetchOpenSession(evt) => self.evt_fetch_open_session(evt),
            Event::FetchListSessions(evt) => self.evt_fetch_list_sessions(evt),
            Event::CheckSession => self.check_list_sessions(),
            Event::ListSesssions { resp } => self.verify_session(resp),
        };

        tracing::trace!("HANDLE EVENT [state after]: {}", self);
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
                            tracing::error!("failed to parse addresses: {:?}", e);
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
                        let mut backoffs: Vec<time::Duration> =
                            entry_node::addressses_backoff()
                                .into_iter()
                                .fold(Vec::new(), |mut acc, e| {
                                    if let Some(dur) = e {
                                        acc.push(dur);
                                    }
                                    acc
                                });
                        backoffs.reverse();
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
                tracing::info!("opened session {}", value);
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
                        tracing::error!("failed to parse session: {:?}", e);
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
                        let mut backoffs: Vec<time::Duration> =
                            session::open_session_backoff()
                                .into_iter()
                                .fold(Vec::new(), |mut acc, e| {
                                    if let Some(dur) = e {
                                        acc.push(dur);
                                    }
                                    acc
                                });
                        backoffs.reverse();
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
                self.fetch_data.addresses = RemoteData::Success;
                tracing::info!("listing sessions {}", value);
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
                        let mut backoffs: Vec<time::Duration> =
                            entry_node::list_sessions_backoff()
                                .into_iter()
                                .fold(Vec::new(), |mut acc, e| {
                                    if let Some(dur) = e {
                                        acc.push(dur);
                                    }
                                    acc
                                });
                        backoffs.reverse();
                        self.repeat_fetch_list_sessions(err, &mut backoffs);
                    }
                    _ => {
                        // should not happen
                        tracing::error!("unexpected event state");
                    }
                }
            }
            remote_data::Event::Retry => self.fetch_list_sessions()?,
        };
        Ok(())
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
        }
    }

    fn repeat_fetch_list_sessions(&mut self, error: remote_data::CustomError, backoffs: &mut Vec<time::Duration>) {
        if let Some(backoff) = backoffs.pop() {
            let cancel_sender = entry_node::schedule_retry_list_sessions(backoff, &self.sender);
            self.fetch_data.list_sessions = RemoteData::RetryFetching {
                error,
                cancel_sender,
                backoffs: backoffs.clone(),
            };
        } else {
            self.fetch_data.list_sessions = RemoteData::Failure { error };
        }
    }

    fn status(&self) -> anyhow::Result<Option<String>> {
        Ok(Some(self.to_string()))
    }

    fn entry_node(
        &mut self,
        endpoint: Url,
        api_token: String,
        session_port: Option<u16>,
    ) -> anyhow::Result<Option<String>> {
        self.cancel_fetch_addresses();
        self.cancel_fetch_open_session();
        self.cancel_fetch_list_sessions();
        self.entry_node = Some(EntryNode::new(endpoint, api_token, session_port));
        self.fetch_data.addresses = RemoteData::Fetching {
            started_at: SystemTime::now(),
        };
        self.fetch_addresses()?;
        self.check_open_session()?;
        Ok(None)
    }

    fn exit_node(&mut self, peer_id: String) -> anyhow::Result<Option<String>> {
        self.cancel_fetch_open_session();
        self.cancel_fetch_list_sessions();
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

    fn check_list_sessions(&mut self) -> anyhow::Result<()> {
        /*
        match (&self.status, &self.entry_node) {
            (Status::MonitoringSession { start_time }, Some(entry_node)) => {
                if start_time.elapsed().unwrap().as_secs() > 3 {
                    self.status = Status::ListingSessions {
                        start_time: SystemTime::now(),
                    };
                    // self.list_sessions(entry_node)
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
        */
        Ok(())
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

    fn verify_session(&mut self, resp: Vec<event::ListSessionsEntry>) -> anyhow::Result<()> {
        /*
        let session_listed = resp.iter().any(|entry| {
            entry
                .target
                .eq_ignore_ascii_case("wireguard.staging.hoprnet.link:51820")
                && entry.protocol.eq_ignore_ascii_case("udp")
                && entry.port == 60006
        });

        if session_listed {
            tracing::info!("session verified open");
            self.status = Status::MonitoringSession {
                start_time: SystemTime::now(),
            };
            self.check_list_sessions()
        } else {
            tracing::info!("session no longer open");
            self.status = Status::Idle;
            self.check_open_session()
        }
        */
        Ok(())
    }

    fn cancel_fetch_addresses(&self) {
        if let RemoteData::RetryFetching { cancel_sender, .. } = &self.fetch_data.addresses {
            let res = cancel_sender.send(());
            match res {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("sending cancel event failed: {:?}", e);
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
                    tracing::warn!("sending cancel event failed: {:?}", e);
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
                    tracing::warn!("sending cancel event failed: {:?}", e);
                }
            }
        }
    }
}

impl fmt::Display for ExitNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let print = HashMap::from([("peer_id", self.peer_id.as_str())]);
        let val = serde_json::to_string(&print).unwrap();
        write!(f, "{}", val)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let val = match self {
            Status::Idle => "idle",
            // TODO refac to remote data
            Status::OpeningSession { start_time } => {
                &format!("opening session for {}", start_time.elapsed().unwrap().as_secs()).to_string()
            }
            Status::MonitoringSession { start_time, .. } => {
                &format!("monitoring session for {}s", start_time.elapsed().unwrap().as_secs()).to_string()
            }
            Status::ListingSessions { start_time } => {
                &format!("listing sessions for {}", start_time.elapsed().unwrap().as_secs()).to_string()
            }
        };
        write!(f, "{}", val)
    }
}

impl fmt::Display for Core {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let en_str: String = self
            .entry_node
            .as_ref()
            .map(|en| en.to_string())
            .unwrap_or("".to_string());
        let xn_str: String = self
            .exit_node
            .as_ref()
            .map(|xn| xn.to_string())
            .unwrap_or("".to_string());
        let print = HashMap::from([
            ("status", self.status.to_string()),
            ("entry_node", en_str),
            ("exit_node", xn_str),
        ]);
        let val = serde_json::to_string(&print).unwrap();
        write!(f, "{}", val)
    }
}
