use crate::{Error, Statement, DataSource};
use std::time::Duration;

pub trait Session {
    fn register_source(&self, ds: Box<dyn DataSource>) -> Result<(),Error>;
    fn query(&self, script: &str, timeout: Duration) -> Result<Statement, Error>;
    fn update(&self, script: &str, timeout: Duration) -> Result<(), Error>;
}

