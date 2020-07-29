use crate::{Request, Columns,Result, Register};

pub trait DataSource: Send + Sync {
    fn name(&self) -> &str;
    fn args(&self) -> Columns;
    fn columns(&self) -> Columns;
    fn get_register(&self) -> &Register;
    fn collect(&self,request: &mut Request) -> Result<()>;
}

#[macro_export]
macro_rules! register_ds {
    ($namespace: ident) => {
        {
            Box::new($namespace::DataSourceImpl::new())
        }
    };
}

#[macro_export]
macro_rules! register_state {
    ($ds: expr, $state: expr) => {
        {
            $ds.get_register().set_state($state);
        }
    };
}