use crate::{Statement, DataSource, Result, Args, Value};
use std::time::Duration;

pub trait Connection {
    fn register_source(&self, ds: Box<dyn DataSource>) -> Result<()>;
    fn register_func<F: 'static, V: Into<Value>>(&self, func: F) -> Result<()> where F: Fn(&Args) -> Result<V>;
    fn register_state<T: Sync + Send + Clone + 'static>(&self, state: T) -> crate::Result<()>;
    fn new_statement(&self, script: &str, timeout: Duration) -> Result<Statement>;
}