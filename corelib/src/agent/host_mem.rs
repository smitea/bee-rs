use crate::{Columns, Promise, Result, Row, ToData};
use async_std::task::block_on;
use heim::memory::{memory, Memory};

#[derive(Data)]
pub struct MemoryUsage {
    used_bytes: i64,
    total_bytes: i64,
    free_bytes: i64,
}

#[datasource]
pub fn memory_usage(promise: &mut Promise<MemoryUsage>) -> Result<()> {
    let memory: Memory = block_on(memory())?;

    let total: u64 = memory.total();
    let available: u64 = memory.available();
    promise.commit(MemoryUsage {
        used_bytes: (total - available) as i64,
        total_bytes: total as i64,
        free_bytes: available as i64,
    })?;
    Ok(())
}
