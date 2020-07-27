use crate::{Request, Columns,Result};

pub trait DataSource: Send + Sync {
    fn name(&self) -> &str;
    fn args(&self) -> Columns;
    fn columns(&self) -> Columns;
    fn collect(&self, request: &mut Request) -> Result<()>;
}