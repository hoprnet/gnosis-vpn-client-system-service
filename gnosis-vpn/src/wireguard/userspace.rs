use crate::task::Task;

#[derive(Debug)]
pub struct UserSpace {
    available: bool,
    error: Option<anyhow::Error>,
}

impl UserSpace {
    pub fn new() -> Self {
        UserSpace {
            available: false,
            error: None,
        }
    }
}

impl Task for UserSpace {
    fn init(&mut self) {}

    fn run(&self) {}
}
