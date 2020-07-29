use crate::{Columns, Promise, Result, Row, ToData};
use async_std::task::block_on;
use heim::host::{platform, Platform};

#[derive(Data)]
pub struct OSInfo {
    os_type: String,
    version: String,
    host_name: String,
}

#[datasource]
pub fn os_info(promise: &mut Promise<OSInfo>) -> Result<()> {
    let platform: Platform = block_on(platform())?;

    promise.commit(OSInfo {
        os_type: platform.system().to_string(),
        version: platform.release().to_string(),
        host_name: platform.hostname().to_string(),
    })?;
    Ok(())
}
