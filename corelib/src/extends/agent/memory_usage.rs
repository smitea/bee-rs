use crate::{columns, row, DataSource, Error, Promise, State};
use async_std::task::block_on;
use heim::memory::{memory, Memory};

pub struct MemoryUsage;

impl DataSource for MemoryUsage {
    fn name(&self) -> &str {
        "memory_usage"
    }
    fn columns(&self) -> crate::Columns {
        columns![
            Integer: "used_bytes",
            Integer: "total_bytes",
            Integer: "free_bytes"
        ]
    }
    fn collect(&self, promise: &mut Promise) -> Result<(), Error> {
        let memory: Memory = block_on(memory())?;

        let total: u64 = memory.total();
        let available: u64 = memory.available();
        promise.commit(State::from(row![
            (total - available) as i64,
            total as i64,
            available as i64
        ]))?;
        Ok(())
    }
}
