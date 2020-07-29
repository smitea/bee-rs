use crate::{Instance, Connection, Result};

mod mkdir;
mod read;
mod upload;

pub fn register_ds<T: Connection>(_: &Instance, connection: &T) -> Result<()> {
    connection.register_source(register_ds!(read))?;
    connection.register_source(register_ds!(mkdir))?;
    connection.register_source(register_ds!(upload))?;
    Ok(())
}