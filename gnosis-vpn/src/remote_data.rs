use exponential_backoff::Backoff;
use reqwest::header;
use reqwest::header::{HeaderMap, HeaderValue};
use std::time::SystemTime;
use crate::event;
use std::vec::Vec;
use std::time;

pub enum RemoteData<E, R> {
    NotAsked,
    Fetching { started_at: SystemTime },
    RetryFetching { error: E, backoffs: Vec<time::Duration> }, // reverse order
    Failure { error: E },
    Success(R),
}

pub enum Event<R> {
    Response(R),
    Retry,
    Error(reqwest::Error),
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
