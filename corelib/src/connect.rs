use crate::{Statement, DataSource, Result, Args, Value};
use std::{panic::UnwindSafe, time::Duration};

pub trait Connection {
    fn register_source(&self, ds: Box<dyn DataSource>) -> Result<()>;
    fn register_func<F, V: Into<Value>>(&self, name: &str, args: usize, func: F) -> Result<()> where F: Fn(&Args) -> Result<V> + Send + UnwindSafe + 'static;
    fn new_statement(&self, script: &str, timeout: Duration) -> Result<Statement>;
}

#[macro_export]
macro_rules! register_func {
    ($connect: expr, $namespace: ident) => {
        let name = $namespace::FunctionImpl::name();
        let args_size =  $namespace::FunctionImpl::args();
        $connect.register_func(name,args_size, $namespace::FunctionImpl::invoke)?;
    };
}