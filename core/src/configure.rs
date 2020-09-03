use crate::{Args, DataSource, Result, Value};
use std::panic::UnwindSafe;

pub trait Configure {
    /// 注册数据源
    fn register_source(&mut self, ds: Box<dyn DataSource>) -> Result<()>;
    /// 注册扩展函数
    fn register_func<F, V: Into<Value>>(&mut self, name: &str, args: usize, func: F) -> Result<()>
    where
        F: Fn(&Args) -> Result<V> + Send + UnwindSafe + Sync + 'static;
}
