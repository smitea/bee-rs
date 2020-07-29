use crate::{Columns, Promise, Result, Row, ToData};
use async_std::task::block_on;
use heim::memory::{swap, Swap};

#[derive(Data)]
pub struct SWAPUsage {
    used_bytes: i64,
    total_bytes: i64,
    free_bytes: i64,
}

#[datasource]
pub fn swap_usage(promise: &mut Promise<SWAPUsage>) -> Result<()> {
    let swap: Swap = block_on(swap())?;

    let total = swap.total();
    let free = swap.free();
    promise.commit(SWAPUsage {
        used_bytes: (total - free) as i64,
        total_bytes: total as i64,
        free_bytes: free as i64,
    })?;
    Ok(())
}
