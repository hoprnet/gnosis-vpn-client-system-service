use std::fmt::Debug;

pub trait Task: Debug {
    fn init(&mut self);
    fn run(&self);
}
