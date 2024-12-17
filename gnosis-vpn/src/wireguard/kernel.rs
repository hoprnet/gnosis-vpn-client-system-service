use crate::task::Task;

// This will be the implementation using netlink kernel access.
#[derive(Debug)]
pub struct Kernel {
    available: bool,
    error: Option<anyhow::Error>,
}

impl Kernel {
    pub fn new() -> Self {
        Kernel {
            available: false,
            error: None,
        }
    }
}

impl Task for Kernel {
    fn init(&mut self) {}

    fn run(&self) {}
}
