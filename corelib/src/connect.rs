use crate::{Statement, DataSource, Result, Args, Value};

pub trait Connection {
    fn register_source(&self, ds: dyn DataSource) -> Result<()>;
    fn register_func<F: 'static, V: Into<Value>>(&self, func: F) -> Result<()> where F: Fn(&Args) -> Result<V>;
    fn new_statement(&self, script: &str) -> Result<Statement>;
}