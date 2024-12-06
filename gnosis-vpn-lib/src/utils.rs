use serde::ser::Serialize;

pub fn display_serialize<T: ?Sized>(v: &T) -> String
where
    T: Serialize,
{
    match serde_json::to_string(&v) {
        Ok(s) => s,
        Err(e) => format!("serializion error: {}", e),
    }
}
