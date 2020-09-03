use crate::{Columns, Promise, Result, Row, ToData};
use heim::memory::{memory, Memory};

#[derive(Data)]
pub struct MemoryUsage {
    used_bytes: i64,
    total_bytes: i64,
    free_bytes: i64,
}

#[datasource]
pub async fn memory_usage(promise: &mut Promise<'_, MemoryUsage>) -> Result<()> {
    let memory: Memory = memory().await?;

    let total: u64 = memory.total();
    let available: u64 = memory.available();
    promise
        .commit(MemoryUsage {
            used_bytes: (total - available) as i64,
            total_bytes: total as i64,
            free_bytes: available as i64,
        })
        .await?;
    Ok(())
}

#[test]
fn test() {
    use crate::*;
    smol::block_on(async {
        let (req, resp) = crate::new_req(crate::Args::new());
        smol::spawn(async move {
            let mut promise = req.head::<MemoryUsage>().await.unwrap();
            if let Err(err) = memory_usage(&mut promise).await {
                let _ = req.error(err);
            } else {
                let _ = req.ok();
            }
        }).detach();

        let resp = resp.wait().await.unwrap();
        assert_eq!(
            &columns![Integer: "used_bytes", Integer: "total_bytes", Integer: "free_bytes"],
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
