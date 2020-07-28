use crate::{Instance, Connection,Result};

pub mod file;
pub mod sftp;

pub fn register_ds<T: Connection>(instance: &Instance,connection: &T) -> Result<()>{
    Ok(())
}