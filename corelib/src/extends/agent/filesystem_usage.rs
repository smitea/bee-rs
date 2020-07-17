use crate::{columns, row, DataSource, Error, Promise, State, Columns};

use async_std::prelude::*;
use async_std::task::block_on;
use heim::disk::partitions_physical;
use std::ffi::OsStr;

pub struct FileSystemUsage;

impl DataSource for FileSystemUsage {
    fn name(&self) -> &str {
        "filesystem_usage"
    }

    fn args(&self) -> Columns {
        columns![]
    }

    fn columns(&self) -> Columns {
        columns![
            String  : "filesystem",
            Integer : "used_bytes",
            Integer : "total_bytes",
            Integer : "available_bytes",
            String  : "mounted_on"
        ]
    }
    fn collect(&self, promise: &mut Promise) -> Result<(), Error> {
        block_on(async {
            let mut partitions = partitions_physical();
            while let Some(Ok(part)) = partitions.next().await {
                if let Ok(usage) = heim::disk::usage(part.mount_point().to_path_buf()).await {
                    let total = usage.total();
                    if total == 0 {
                        break;
                    }
                    let filesystem: String = format!(
                        "{}",
                        part.device()
                            .unwrap_or_else(|| OsStr::new("N/A"))
                            .to_string_lossy()
                    );
                    let mount_on = format!("{}", part.mount_point().to_string_lossy());
                    if let Err(_) = promise.commit(State::from(row![
                        filesystem,
                        usage.used() as i64,
                        usage.total() as i64,
                        usage.free() as i64,
                        mount_on
                    ])) {
                        break;
                    }
                }
            }
        });
        Ok(())
    }
}
