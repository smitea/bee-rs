use crate::{Instance, Result, Configure};

pub mod func_get;
pub mod split_csv;
pub mod split_space;

pub async fn register_ds<T: Configure>(_instance: &Instance, connection: &mut T) -> Result<()> {
    crate::register_func!(connection, func_get);
    crate::register_func!(connection, split_csv);
    crate::register_func!(connection, split_space);
    Ok(())
}
