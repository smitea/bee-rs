use crate::Session;

pub trait Connection {
    fn connect(&self, url: &str) -> dyn Session;
}
