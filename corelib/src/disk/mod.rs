use crate::{Columns, Connection, Instance, Result, Row};

#[cfg(feature = "agent")]
mod agent;
#[cfg(feature = "remote")]
mod remote;

#[derive(Data)]
pub struct FileLine {
    pub line: String,
    pub line_num: u32,
}

#[derive(Data)]
pub struct Status {
    pub success: bool
}

pub fn register_ds<T: Connection>(instance: &Instance, connection: &T) -> Result<()> {
    #[cfg(feature = "agent")]
    agent::register_ds(instance, connection)?;
    #[cfg(feature = "remote")]
    remote::register_ds(instance, connection)?;
    Ok(())
}
