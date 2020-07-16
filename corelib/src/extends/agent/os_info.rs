use crate::{columns, row, Columns, DataSource, Error, Promise, State};
use async_std::task::block_on;
use heim::host::{platform, Platform};

pub struct OSInfo;

impl DataSource for OSInfo {
    fn name(&self) -> &str {
        "os_info"
    }
    fn columns(&self) -> Columns {
        columns![
            String : "os_type",
            String : "version",
            String : "host_name"
        ]
    }
    fn collect(&self, promise: &mut Promise) -> Result<(), Error> {
        let platform: Platform = block_on(platform())?;

        promise.commit(State::from(row![
            platform.system(),
            platform.release(),
            platform.hostname()
        ]))?;
        
        Ok(())
    }
}
