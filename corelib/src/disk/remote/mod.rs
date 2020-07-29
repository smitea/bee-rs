use crate::{Connection, DataSource, Instance, Result};

mod mkdir;
mod read;
mod upload;

pub fn register_ds<T: Connection>(instance: &Instance, connection: &T) -> Result<()> {
    let session = crate::bash::remote::new_session(instance)?;

    let ds = register_ds!(read);
    register_state!(ds, session.clone());
    let ds = register_ds!(upload);
    register_state!(ds, session.clone());
    let ds = register_ds!(mkdir);
    register_state!(ds, session.clone());
    connection.register_source(ds)?;
    Ok(())
}
