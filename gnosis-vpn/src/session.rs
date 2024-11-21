
pub trait Session {
    fn open(&self) -> anyhow::Result<()>;
    fn close(&self) -> anyhow::Result<()>;
    fn is_active(&self) -> anyhow::Result<bool>;
}


pub struct BasicSession {
    // TODO: add long lived HTTP session to the target API
}

impl BasicSession {
    pub fn new() -> BasicSession {
        BasicSession {}
    }
}
