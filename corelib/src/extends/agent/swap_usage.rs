use crate::{columns, row, DataSource, State};
use async_std::task::block_on;
use heim::memory::{swap, Swap};

pub struct SWAPUsage;

impl DataSource for SWAPUsage {
    fn name(&self) -> &str {
        "swap_usage"
    }
    fn columns(&self) -> crate::Columns {
        columns![
            Integer : "used_bytes",
            Integer : "total_bytes",
            Integer : "free_bytes"
        ]
    }
    fn collect(&self, promise: &mut crate::Promise) -> Result<(), crate::Error> {
        let swap: Swap = block_on(swap())?;

        let total = swap.total();
        let free = swap.free();
        promise.commit(State::from(row![
            (total - free) as i64,
            total as i64,
            free as i64
        ]))?;
        Ok(())
    }
}
