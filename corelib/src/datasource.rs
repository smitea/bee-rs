use crate::{Error, Request, Instance};

pub trait DataSource: Send + Sync{
    fn name(&self) -> &str;
    fn collect(&self, request: &Request) -> Result<(), Error>;
}

pub type Driver = fn (instance: &Instance) -> Result<Box<dyn DataSource>,Error>;