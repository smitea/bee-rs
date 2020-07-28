use crate::{Columns, Error, Promise, Row, ToData};

use super::{format, run_command};
use async_std::task::block_on;
use heim::{
    host::{platform, uptime, Platform},
    memory::{memory, Memory},
};

#[cfg(target_os = "windows")]
const BRAND_CMD: &str = "WMIC CPU Get Name / Format:List 2>nul";
#[cfg(target_os = "macos")]
const BRAND_CMD: &str = "sysctl -a |grep \"machdep.cpu.brand_string\" |awk -F \":\" '{print $2}'";
#[cfg(target_os = "linux")]
const BRAND_CMD: &str = "cat /proc/cpuinfo |grep \"model name\" | awk -F\":\" 'NR==1{print $2}'";

fn cpu_brand() -> Result<String, Error> {
    let output = run_command(BRAND_CMD)?;
    let rs: String = if cfg!(target_os = "windows") {
        output
            .split("=")
            .skip(1)
            .next()
            .map(|val| val.trim())
            .unwrap_or("")
            .to_owned()
    } else {
        output.trim().to_owned()
    };

    return Ok(rs);
}

#[derive(Data)]
pub struct HostBasic {
    pub host_name: String,
    pub cpu_core: i64,
    pub cpu_model: String,
    pub uptime: i64,
    pub memory: i64,
}

#[datasource]
pub fn host_basic(promise: &mut Promise<HostBasic>) -> Result<(), Error> {
    let platform: Platform = block_on(platform())?;
    let uptime: i64 = format(block_on(uptime())?) as i64;
    let cpu_core: i64 = num_cpus::get() as i64;
    let memory: Memory = block_on(memory())?;

    let mem_size: i64 = memory.total() as i64;
    let cpu_brand = cpu_brand()?;

    promise.commit(HostBasic {
        host_name: platform.hostname().to_string(),
        cpu_core,
        cpu_model: cpu_brand,
        uptime,
        memory: mem_size,
    })?;

    Ok(())
}
