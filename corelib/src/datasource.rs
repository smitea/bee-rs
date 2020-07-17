use crate::{Error, Instance, Columns, Promise};

pub trait DataSource: Send + Sync{
    fn name(&self) -> &str;
    fn columns(&self) -> Columns;
    fn args(&self) -> Columns;
    fn collect(&self, promise: &mut Promise) -> Result<(), Error>;
}

pub type Driver = fn (instance: &Instance) -> Result<Box<dyn DataSource>,Error>;