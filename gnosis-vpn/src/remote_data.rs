use exponential_backoff::Backoff;
use reqwest::header;
use reqwest::header::{HeaderMap, HeaderValue};
use std::time::SystemTime;

pub enum RemoteData<E, R> {
    NotAsked,
    Fetching { started_at: SystemTime },
    Failure { error: E, backoff: Backoff },
    Success(R),
}

pub enum ResultEvent<R> {
    Response(R),
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

impl std::fmt::Display for ResultEvent<serde_json::Value> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ResultEvent::Response(val) => write!(f, "Response: {:?}", val),
            ResultEvent::Error(e) => write!(f, "Error: {:?}", e),
        }
    }
}
