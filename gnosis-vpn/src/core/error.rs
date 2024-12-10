#[derive(Debug)]
pub enum Error {
    JsonParseError(serde_json::Error),
    UnexpectedInternalState(String),
}
