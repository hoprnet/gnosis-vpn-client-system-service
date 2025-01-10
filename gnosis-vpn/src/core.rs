use anyhow::Result;
use gnosis_vpn_lib::command::Command;
use gnosis_vpn_lib::config::Config;
use gnosis_vpn_lib::peer_id::PeerId;
use gnosis_vpn_lib::state::State;
use gnosis_vpn_lib::{config, log_output, state, wireguard};
use rand::Rng;
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
use crate::event::Event;
use crate::exit_node::ExitNode;
use crate::remote_data;
use crate::remote_data::RemoteData;
use crate::session;
use crate::session::Session;

#[derive(Debug)]
pub struct Core {
    // http client
    client: blocking::Client,
    // configuration data
    config: Config,
    // event transmitter
    sender: crossbeam_channel::Sender<Event>,
    // potential non critial user visible errors
    issues: Vec<Issue>,
    // internal persistent application state
    state: state::State,
    // wg interface,
    wg: Option<Box<dyn wireguard::WireGuard>>,
    // random generator
    rng: rand::rngs::ThreadRng,

    status: Status,
    entry_node: Option<EntryNode>,
    exit_node: Option<ExitNode>,
    fetch_data: FetchData,
    session: Option<Session>,
}

#[derive(Debug)]
struct FetchData {
    addresses: RemoteData,
    open_session: RemoteData,
    list_sessions: RemoteData,
    close_session: RemoteData,
}

#[derive(Debug)]
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

#[derive(Debug)]
enum Issue {
    Config(config::Error),
    State(state::Error),
    WireGuardInit(wireguard::Error),
    WireGuard(wireguard::Error),
}

fn read_config() -> (Config, Option<Issue>) {
    match config::read() {
        Ok(cfg) => {
            tracing::info!("read config without issues");
            (cfg, None)
        }
        Err(config::Error::NoFile) => {
            tracing::info!("no config - using default");
            (Config::default(), None)
        }
        Err(err) => {
            tracing::warn!(warn = ?err, "failed to read config file");
            (Config::default(), Some(Issue::Config(err)))
        }
    }
}

fn read_state() -> (State, Option<Issue>) {
    match state::read() {
        Ok(state) => (state, None),
        Err(state::Error::NoFile) => (State::default(), None),
        Err(err) => {
            tracing::warn!(warn = ?err, "failed to read state file");
            (State::default(), Some(Issue::State(err)))
        }
    }
}

impl Core {
    pub fn init(sender: crossbeam_channel::Sender<Event>) -> Core {
        let (config, conf_issue) = read_config();
        let mut issues = conf_issue.map(|i| vec![i]).unwrap_or(Vec::new());
        let (wg, wg_errors) = wireguard::best_flavor();
        let mut wg_issues = wg_errors.iter().map(|i| Issue::WireGuardInit(i.clone())).collect();
        issues.append(&mut wg_issues);
        let (state, state_issue) = read_state();
        if let Some(issue) = state_issue {
            issues.push(issue);
        }

        let mut core = Core {
            config,
            issues,
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
            state,
            wg,
            rng: rand::thread_rng(),
            sender,
            session: None,
        };
        core.setup();
        core
    }

    fn setup(&mut self) {
        self.setup_wg_priv_key();
        if let Err(err) = self.setup_from_config() {
            tracing::error!(?err, "failed setup from config");
        }
    }

    fn wg_priv_key(&self) -> Option<&str> {
        if let Some(key) = &self.config.wire_guard.as_ref().map(|wg| wg.private_key.as_str()) {
            return Some(key);
        }
        if let Some(key) = &self.state.wg_private_key {
            return Some(key);
        }
        return None;
    }

    fn setup_wg_priv_key(&mut self) {
        // if wg is available check private key
        // gengerate a new one if none
        if let (Some(wg), None) = (&self.wg, &self.wg_priv_key()) {
            let priv_key = match wg.generate_key() {
                Ok(priv_key) => priv_key,
                Err(err) => {
                    tracing::error!(?err, "failed to generate wireguard private key");
                    self.replace_issue(Issue::WireGuard(err));
                    return;
                }
            };
            match self.state.set_wg_private_key(priv_key) {
                Ok(_) => (),
                Err(err) => {
                    tracing::error!(?err, "failed to write wireguard private key to state");
                    self.replace_issue(Issue::State(err));
                }
            };
        }
    }

    fn setup_from_config(&mut self) -> Result<()> {
        self.check_close_session()?;
        if let (Some(entry_node), Some(session)) = (&self.config.entry_node, &self.config.session) {
            let en_endpoint = entry_node.endpoint.clone();
            let en_api_token = entry_node.api_token.clone();
            let en_listen_host = session.listen_host.clone();
            let path = session.path.clone().unwrap_or_default();
            let en_path = match path {
                config::SessionPathConfig::Hop(hop) => Path::Hop(hop),
                config::SessionPathConfig::IntermediateId(id) => Path::IntermediateId(id),
            };
            let xn_peer_id = session.destination;

            // convert config to old application struture
            self.entry_node = Some(EntryNode::new(
                &en_endpoint,
                &en_api_token,
                en_listen_host.as_deref(),
                en_path,
            ));
            self.exit_node = Some(ExitNode { peer_id: xn_peer_id });

            self.fetch_data.addresses = RemoteData::Fetching {
                started_at: SystemTime::now(),
            };
            self.fetch_addresses()?;
            self.check_open_session()?;
        }
        Ok(())
    }

    #[instrument(level = tracing::Level::INFO, ret(level = tracing::Level::DEBUG))]
    pub fn handle_cmd(&mut self, cmd: &Command) -> Result<Option<String>> {
        match cmd {
            Command::Status => Ok(self.status()),
            Command::EntryNode {
                endpoint,
                api_token,
                listen_host,
                hop,
                intermediate_id,
            } => self.entry_node(endpoint, api_token, listen_host, hop, intermediate_id),
            Command::ExitNode { peer_id } => self.exit_node(peer_id),
        }
    }

    #[instrument(level = tracing::Level::INFO, ret(level = tracing::Level::DEBUG))]
    pub fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::FetchAddresses(evt) => self.evt_fetch_addresses(evt),
            Event::FetchOpenSession(evt) => self.evt_fetch_open_session(evt),
            Event::FetchListSessions(evt) => self.evt_fetch_list_sessions(evt),
            Event::FetchCloseSession(evt) => self.evt_fetch_close_session(evt),
            Event::CheckSession => self.evt_check_session(),
        }
    }

    #[instrument(level = tracing::Level::INFO, ret(level = tracing::Level::DEBUG))]
    pub fn update_config(&mut self) {
        let (config, issue) = read_config();
        self.config = config;
        if let Some(issue) = issue {
            self.replace_issue(issue);
        }
        self.setup();
    }

    fn replace_issue(&mut self, issue: Issue) {
        // remove existing config issue
        self.issues.retain(|i| match (i, &issue) {
            (Issue::Config(_), Issue::Config(_)) => false,
            (Issue::WireGuard(_), Issue::WireGuard(_)) => false,
            (Issue::State(_), Issue::State(_)) => false,
            _ => true,
        });
        self.issues.push(issue);
    }

    fn evt_fetch_addresses(&mut self, evt: remote_data::Event) -> Result<()> {
        match evt {
            remote_data::Event::Response(value) => {
                self.fetch_data.addresses = RemoteData::Success;
                match &mut self.entry_node {
                    Some(en) => {
                        let addresses = serde_json::from_value::<entry_node::Addresses>(value)?;
                        en.addresses = Some(addresses);
                        Ok(())
                    }
                    None => anyhow::bail!("unexpected internal state: no entry node"),
                }
            }
            remote_data::Event::Error(err) => match &self.fetch_data.addresses {
                RemoteData::RetryFetching {
                    backoffs: old_backoffs, ..
                } => {
                    let mut backoffs = old_backoffs.clone();
                    self.repeat_fetch_addresses(err, &mut backoffs);
                    Ok(())
                }
                RemoteData::Fetching { .. } => {
                    let mut backoffs = backoff::get_addresses().to_vec();
                    self.repeat_fetch_addresses(err, &mut backoffs);
                    Ok(())
                }
                _ => anyhow::bail!("unexpected internal state: remote data result while not fetching"),
            },
            remote_data::Event::Retry => self.fetch_addresses(),
        }
    }

    fn evt_fetch_open_session(&mut self, evt: remote_data::Event) -> Result<()> {
        match evt {
            remote_data::Event::Response(value) => {
                let session = serde_json::from_value::<Session>(value)?;
                self.fetch_data.open_session = RemoteData::Success;
                self.session = Some(session);
                let next_check = self.rng.gen_range(5..13);
                let cancel_sender =
                    session::schedule_check_session(time::Duration::from_secs(next_check), &self.sender);
                self.status = Status::MonitoringSession {
                    start_time: SystemTime::now(),
                    cancel_sender,
                };

                // connect wireguard session if possible
                if let (Some(wg), Some(privkey), Some(wg_conf), Some(sess_conf), Some(en_conf)) = (
                    &self.wg,
                    &self.wg_priv_key(),
                    &self.config.wireguard,
                    &self.config.session,
                    &self.config.entry_node,
                ) {
                    if let Some(en_host) = en_conf.endpoint.host() {
                        let info = wireguard::SessionInfo::new(
                            &privkey,
                            wg_conf.address.as_str(),
                            sess_conf.destination.to_string().as_str(),
                            format!("{}:{}", en_host, session.port).as_str(),
                        );

                        match self.wg.connect_session(&info) {
                            Ok(_) => (),
                            Err(err) => {
                                tracing::error!(?err, "failed to connect wireguard session");
                                self.replace_issue(Issue::WireGuard(err));
                            }
                        }
                    }
                }
                Ok(())
            }
            remote_data::Event::Error(err) => match &self.fetch_data.open_session {
                RemoteData::RetryFetching {
                    backoffs: old_backoffs, ..
                } => {
                    let mut backoffs = old_backoffs.clone();
                    self.repeat_fetch_open_session(err, &mut backoffs);
                    Ok(())
                }
                RemoteData::Fetching { .. } => {
                    let mut backoffs = backoff::open_session().to_vec();
                    self.repeat_fetch_open_session(err, &mut backoffs);
                    Ok(())
                }
                _ => anyhow::bail!("unexpected internal state: remote data result while not fetching"),
            },
            remote_data::Event::Retry => self.fetch_open_session(),
        }
    }

    fn evt_fetch_list_sessions(&mut self, evt: remote_data::Event) -> Result<()> {
        match evt {
            remote_data::Event::Response(value) => {
                self.fetch_data.list_sessions = RemoteData::Success;
                let res_sessions = serde_json::from_value::<Vec<session::Session>>(value);
                match res_sessions {
                    Ok(sessions) => self.verify_session(&sessions),
                    Err(e) => {
                        tracing::warn!("stopped monitoring - failed to parse sessions");
                        self.status = Status::Idle;
                        anyhow::bail!("failed to parse sessions: {}", e);
                    }
                }
            }
            remote_data::Event::Error(err) => match &self.fetch_data.list_sessions {
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
                _ => anyhow::bail!("unexpected internal state: remote data result while not fetching"),
            },
            remote_data::Event::Retry => self.fetch_list_sessions(),
        }
    }

    fn evt_fetch_close_session(&mut self, evt: remote_data::Event) -> Result<()> {
        match evt {
            remote_data::Event::Response(_) => {
                self.fetch_data.close_session = RemoteData::Success;
                self.status = Status::Idle;
                self.check_open_session()
            }
            remote_data::Event::Error(err) => match &self.fetch_data.close_session {
                RemoteData::RetryFetching {
                    backoffs: old_backoffs, ..
                } => {
                    let mut backoffs = old_backoffs.clone();
                    self.repeat_fetch_close_session(err, &mut backoffs)
                }
                RemoteData::Fetching { .. } => {
                    let mut backoffs = backoff::close_session().to_vec();
                    self.repeat_fetch_close_session(err, &mut backoffs)
                }
                _ => anyhow::bail!("unexpected internal state: remote data result while not fetching"),
            },
            remote_data::Event::Retry => self.fetch_close_session(),
        }
    }

    fn evt_check_session(&mut self) -> Result<()> {
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
    ) -> Result<()> {
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
                anyhow::bail!("unexpected internal state: failed list session call while not monitoring session")
            }
        }
    }

    fn repeat_fetch_close_session(
        &mut self,
        error: remote_data::CustomError,
        backoffs: &mut Vec<time::Duration>,
    ) -> Result<()> {
        if let Some(backoff) = backoffs.pop() {
            let cancel_sender = session::schedule_retry_close(backoff, &self.sender);
            self.fetch_data.close_session = RemoteData::RetryFetching {
                error,
                cancel_sender,
                backoffs: backoffs.clone(),
            };
            Ok(())
        } else {
            self.fetch_data.close_session = RemoteData::Failure { error };
            if let Status::ClosingSession { .. } = self.status {
                self.status = Status::Idle;
                Ok(())
            } else {
                anyhow::bail!("unexpected internal state: failed close session call while not closing session")
            }
        }
    }

    fn status(&self) -> Option<String> {
        Some(self.to_string())
    }

    fn entry_node(
        &mut self,
        endpoint: &Url,
        api_token: &str,
        listen_host: &Option<String>,
        hop: &Option<u8>,
        intermediate_id: &Option<PeerId>,
    ) -> Result<Option<String>> {
        self.check_close_session()?;

        // TODO move this to library and enhance CLI to only allow one option
        // hop has precedence over intermediate_id
        let path = match (hop, intermediate_id) {
            (Some(h), _) => Path::Hop(*h),
            (_, Some(id)) => Path::IntermediateId(*id),
            _ => Path::Hop(1),
        };
        self.entry_node = Some(EntryNode::new(endpoint, api_token, listen_host.as_deref(), path));
        self.fetch_data.addresses = RemoteData::Fetching {
            started_at: SystemTime::now(),
        };
        self.fetch_addresses()?;
        self.check_open_session()?;
        Ok(None)
    }

    fn exit_node(&mut self, peer_id: &PeerId) -> Result<Option<String>> {
        self.check_close_session()?;
        self.exit_node = Some(ExitNode { peer_id: *peer_id });
        self.check_open_session()?;
        Ok(None)
    }

    fn check_open_session(&mut self) -> Result<()> {
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

    fn check_close_session(&mut self) -> Result<()> {
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

    fn fetch_addresses(&mut self) -> Result<()> {
        match &self.entry_node {
            Some(en) => en.query_addresses(&self.client, &self.sender),
            _ => Ok(()),
        }
    }

    fn fetch_open_session(&mut self) -> Result<()> {
        match (&self.entry_node, &self.exit_node) {
            (Some(en), Some(xn)) => session::open(&self.client, &self.sender, en, xn),
            _ => Ok(()),
        }
    }

    fn fetch_list_sessions(&mut self) -> Result<()> {
        match &self.entry_node {
            Some(en) => en.list_sessions(&self.client, &self.sender),
            _ => Ok(()),
        }
    }

    fn fetch_close_session(&mut self) -> Result<()> {
        match (&self.entry_node, &self.session) {
            (Some(en), Some(sess)) => sess.close(&self.client, &self.sender, en),
            _ => Ok(()),
        }
    }

    fn verify_session(&mut self, sessions: &[session::Session]) -> Result<()> {
        match (&self.session, &self.status) {
            (Some(sess), Status::MonitoringSession { start_time, .. }) => {
                if sess.verify_open(sessions) {
                    tracing::info!(session = ?sess, since = log_output::elapsed(start_time), "verified session open");
                    let cancel_sender = session::schedule_check_session(time::Duration::from_secs(9), &self.sender);
                    self.status = Status::MonitoringSession {
                        start_time: *start_time,
                        cancel_sender,
                    };
                    Ok(())
                } else {
                    tracing::warn!(session = ?sess, "session no longer open");
                    self.status = Status::Idle;
                    self.check_open_session()
                }
            }
            (Some(_sess), _) => {
                anyhow::bail!("unexpected internal state: session verification while not monitoring session")
            }
            (None, _status) => anyhow::bail!("unexpected internal state: session verification while no session"),
        }
    }

    fn cancel_fetch_addresses(&self) {
        if let RemoteData::RetryFetching { cancel_sender, .. } = &self.fetch_data.addresses {
            let res = cancel_sender.send(());
            match res {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!(error = %e, "failed sending cancel query addresses");
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
                    tracing::warn!(error = %e, "failed sending cancel open session");
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
                    tracing::warn!(error = %e, "failed sending cancel list sessions");
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
                    tracing::warn!(error = %e, "failed sending cancel close session");
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
                    tracing::warn!(error = %e, "failed sending cancel monitoring session");
                }
            }
        }
    }
}

impl fmt::Display for ExitNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let peer = self.peer_id.to_string();
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
        let mut print = HashMap::new();
        if self.config == Config::default() {
            print.insert("config", "<default>".to_string());
        }
        if self.issues.len() > 0 {
            print.insert(
                "issues",
                self.issues
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<String>>()
                    .join(" | "),
            );
        }
        let val = log_output::serialize(&print);
        write!(f, "{}", val)
    }
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let val = match self {
            Issue::Config(e) => format!("config file issue: {}", e),
            Issue::WireGuardInit(e) => format!("wireguard initialization issue: {}", e),
            Issue::State(e) => format!("storage issue: {}", e),
            Issue::WireGuard(e) => format!("wireguard issue: {}", e),
        };
        write!(f, "{}", val)
    }
}
