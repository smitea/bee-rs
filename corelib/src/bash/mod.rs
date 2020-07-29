use crate::{Columns, Connection, DataSource, Instance, Result, Row};

#[cfg(feature = "agent")]
mod local;
#[cfg(feature = "remote")]
pub mod remote;

#[derive(Data)]
pub struct BashRow {
    line: String,
    line_num: u32,
}

pub fn register_ds<T: Connection>(instance: &Instance, connection: &T) -> Result<()> {
    #[cfg(feature = "agent")]
    register_agent_ds(instance, connection)?;
    #[cfg(feature = "remote")]
    register_remote_ds(instance, connection)?;
    Ok(())
}

#[cfg(feature = "agent")]
pub fn register_agent_ds<T: Connection>(_: &Instance, connection: &T) -> Result<()> {
    connection.register_source(register_ds!(local))?;
    Ok(())
}

#[cfg(feature = "remote")]
pub fn register_remote_ds<T: Connection>(instance: &Instance, connection: &T) -> Result<()> {
    let ds = register_ds!(remote);
    register_state!(ds, remote::new_session(instance)?);
    connection.register_source(ds)?;
    Ok(())
}
