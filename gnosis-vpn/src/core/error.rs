use reqwest::header::InvalidHeaderValue;
use url::ParseError;

#[derive(Debug)]
pub enum Error {
    ParseJson(serde_json::Error),
    UnexpectedInternalState(String),
    HeaderSerialization(InvalidHeaderValue),
    Url(ParseError),
}
