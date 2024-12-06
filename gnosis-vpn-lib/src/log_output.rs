use humantime::format_duration;
use serde::ser::Serialize;
use std::time::SystemTime;

pub fn serialize<T: ?Sized>(v: &T) -> String
where
    T: Serialize,
{
    match serde_json::to_string(&v) {
        Ok(s) => s,
        Err(e) => format!("serializion error: {}", e),
    }
}

pub fn elapsed(timestamp: &SystemTime) -> String {
    match timestamp.elapsed() {
        Ok(elapsed) => format_duration(elapsed).to_string(),
        Err(e) => format!("error displaying duration: {}", e),
    }
}
