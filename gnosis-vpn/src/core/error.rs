use crossbeam_channel::SendError;
use reqwest::header::InvalidHeaderValue;
use url::ParseError;

#[derive(Debug)]
pub enum Error {
    JsonParseError(serde_json::Error),
    UnexpectedInternalState(String),
    InvalidHeaderValue(InvalidHeaderValue),
    UrlParseError(ParseError),
    ChannelSendError(SendError<()>),
}
