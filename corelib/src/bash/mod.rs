use crate::{Result, Columns, Row, Connection, Instance};

#[cfg(feature = "agent")]
mod local;
#[cfg(feature = "remote")]
mod remote;

#[derive(Data)]
pub struct BashRow {
    line: String,
    line_num: u32,
}

pub fn register_ds<T: Connection>(instance: &Instance,connection: &T) -> Result<()>{
    Ok(())
}