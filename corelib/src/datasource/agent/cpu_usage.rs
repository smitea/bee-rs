use crate::{Columns, Promise, Result, Row, ToData};
use async_std::task::block_on;
use std::time::Duration;

#[derive(Data)]
pub struct CPUUsage {
    idle: f64,
    user: f64,
    system: f64,
    iowait: f64,
}

#[datasource]
pub fn cpu_usage(promise: &mut Promise<CPUUsage>) -> Result<()> {
    let cpu_usage = block_on(heim::cpu::usage(Duration::from_secs(1)))?;

    promise.commit(CPUUsage {
        idle: cpu_usage.idle(),
        user: cpu_usage.user(),
        system: cpu_usage.system(),
        iowait: 0.0,
    })?;
    Ok(())
}
