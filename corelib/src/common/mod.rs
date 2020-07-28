use crate::{Connection,Result, Instance};

pub mod functions;
#[cfg(feature = "agent")]
pub mod shell_util;

pub fn register_ds<T: Connection>(instance: &Instance,connection: &T) -> Result<()>{
    Ok(())
}