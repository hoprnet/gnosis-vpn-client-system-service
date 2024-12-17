use std::vec;

use crate::task::Task;

mod kernel;
mod tooling;
mod userspace;

pub trait Wireguard {}

pub fn tasks() -> Vec<Box<dyn Task>> {
    vec![
        Box::new(kernel::Kernel::new()),
        Box::new(tooling::Tooling::new()),
        Box::new(userspace::UserSpace::new()),
    ]
}
