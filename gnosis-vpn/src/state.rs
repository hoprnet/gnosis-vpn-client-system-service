use url::Url;

pub struct State {
    status: Status,
    entry_node: Option<EntryNode>,
}

pub enum Status {
    Starting,
    Idle,
    OpenSession,
}

struct EntryNode {
    endpoint: Url,
    api_token: String,
}

impl State {
    pub fn init() -> State {
        State {
            status: Status::Starting,
            entry_node: None,
        }
    }

    pub fn update_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn update_entry_node(&mut self, endpoint: Url, api_token: String) {
        self.entry_node = Some(EntryNode {
            endpoint,
            api_token,
        });
    }

    pub fn to_string(&self) -> String {
        match self.status {
            Status::Starting => "starting".to_string(),
            Status::Idle => "idle".to_string(),
            Status::OpenSession => format!(
                "open session to {}",
                self.entry_node.as_ref().unwrap().endpoint
            ),
        }
    }
}
