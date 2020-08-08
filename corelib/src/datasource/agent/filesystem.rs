use crate::{Columns, Error, Promise, Row, ToData};

use async_std::prelude::*;
use async_std::task::block_on;
use heim::disk::partitions_physical;
use std::ffi::OsStr;

#[derive(Data)]
pub struct Filesystem {
    pub name: String,
    pub mount_on: String,
    pub total_bytes: i64,
    pub used_bytes: i64,
    pub free_bytes: i64,
}

#[datasource]
pub fn filesystem(promise: &mut Promise<Filesystem>) -> Result<(), Error> {
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
                if let Err(_) = promise.commit(Filesystem {
                    name: filesystem,
                    used_bytes: usage.used() as i64,
                    total_bytes: usage.total() as i64,
                    free_bytes: usage.free() as i64,
                    mount_on,
                }) {
                    break;
                }
            }
        }
    });
    Ok(())
}
