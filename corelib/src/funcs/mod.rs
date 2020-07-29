use crate::{Connection, Instance, Result};

pub mod func_get;
pub mod split_csv;
pub mod split_space;

pub fn register_ds<T: Connection>(_instance: &Instance, connection: &T) -> Result<()> {
    register_func!(connection, func_get);
    register_func!(connection, split_csv);
    register_func!(connection, split_space);
    Ok(())
}
