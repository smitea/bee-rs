use crate::{Columns, Error, Promise, Row, ToData};

use heim::disk::partitions_physical;
use smol::prelude::*;
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
pub async fn filesystem(promise: &mut Promise<'_, Filesystem>) -> Result<(), Error> {
    let mut partitions = Box::pin(partitions_physical());
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
            promise
                .commit(Filesystem {
                    name: filesystem,
                    used_bytes: usage.used() as i64,
                    total_bytes: usage.total() as i64,
                    free_bytes: usage.free() as i64,
                    mount_on,
                })
                .await?;
        }
    }
    Ok(())
}

#[test]
fn test() {
    use crate::*;
    smol::block_on(async {
        let (req, resp) = crate::new_req(crate::Args::new());
        smol::spawn(async move{
            let mut promise = req.head::<Filesystem>().await.unwrap();
            if let Err(err) = filesystem(&mut promise).await {
                let _ = req.error(err);
            } else {
                let _ = req.ok();
            }
        }).detach();

        let resp = resp.wait().await.unwrap();
        assert_eq!(
            &columns![String: "name", String: "mount_on", Integer: "total_bytes",Integer: "used_bytes", Integer: "free_bytes"],
            resp.columns()
        );

        let mut index = 0;
        for row in resp {
            let _ = row.unwrap();
            index += 1;
        }
        assert!(index > 0);
    });
}
