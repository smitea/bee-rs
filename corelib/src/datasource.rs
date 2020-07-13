use crate::{Error, Request};

pub trait DataSource: Send + Sync{
    fn name(&self) -> &str;
    fn collect(&self, request: &Request) -> Result<(), Error>;
}
