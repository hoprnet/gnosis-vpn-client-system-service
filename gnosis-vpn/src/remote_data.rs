use reqwest::header;
use reqwest::header::{HeaderMap, HeaderValue};
use std::time;
use std::time::SystemTime;
use std::vec::Vec;

pub enum RemoteData<R> {
    NotAsked,
    Fetching {
        started_at: SystemTime,
    },
    RetryFetching {
        error: CustomError,
        // in reverse order
        backoffs: Vec<time::Duration>,
        cancel_sender: crossbeam_channel::Sender<()>,
    },
    Failure {
        error: CustomError,
    },
    Success(R),
}

#[derive(Debug)]
pub struct CustomError {
    pub reqwErr: Option<reqwest::Error>,
    pub status: Option<reqwest::StatusCode>,
    pub value: Option<serde_json::Value>,
}

pub enum Event<R> {
    Response(R),
    Retry,
    Error(CustomError),
}

pub fn authentication_headers(api_token: &str) -> anyhow::Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    let mut hv_token = HeaderValue::from_str(api_token)?;
    hv_token.set_sensitive(true);
    headers.insert("x-auth-token", hv_token);
    Ok(headers)
}

impl std::fmt::Display for Event<serde_json::Value> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Event::Response(val) => write!(f, "Response: {:?}", val),
            Event::Retry => write!(f, "Retry"),
            Event::Error(e) => write!(f, "Error: {:?}", e),
        }
    }
}
