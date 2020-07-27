use crate::{columns, row, DataSource, Error, Promise, State, Columns};
use async_std::task::block_on;
use std::time::Duration;

pub struct CPUUsage;

impl DataSource for CPUUsage {
    fn name(&self) -> &str {
        "cpu_usage"
    }

    fn columns(&self) -> Columns {
        columns![
            Number  : "idle",
            Number  : "user",
            Number  : "system",
            Number  : "iowait"
        ]
    }

    fn collect(&self, promise: &mut Promise) -> Result<(), Error> {
        let cpu_usage = block_on(heim::cpu::usage(Duration::from_secs(1)))?;

        promise.commit(State::from(row![
            cpu_usage.idle(),
            cpu_usage.user(),
            cpu_usage.system(),
            0
        ]))?;
        Ok(())
    }
}
