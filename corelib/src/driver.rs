use crate::{DataSource, Instance,Result};

pub trait Driver {
    fn name(&self) -> &str;
    fn new_datasource(&self, instance: Instance) -> Result<Box<dyn DataSource>>;
}