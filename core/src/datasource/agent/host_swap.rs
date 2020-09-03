use crate::{Columns, Promise, Result, Row, ToData};
use heim::memory::{swap, Swap};

#[derive(Data)]
pub struct SWAPUsage {
    used_bytes: i64,
    total_bytes: i64,
    free_bytes: i64,
}

#[datasource]
pub async fn swap_usage(promise: &mut Promise<'_, SWAPUsage>) -> Result<()> {
    let swap: Swap = swap().await?;

    let total = swap.total();
    let free = swap.free();
    promise
        .commit(SWAPUsage {
            used_bytes: (total - free) as i64,
            total_bytes: total as i64,
            free_bytes: free as i64,
        })
        .await?;
    Ok(())
}

#[test]
fn test() {
    use crate::*;
    smol::block_on(async {
        let (req, resp) = crate::new_req(crate::Args::new());
        {
            let mut promise = req.head::<SWAPUsage>().await.unwrap();
            if let Err(err) = swap_usage(&mut promise).await {
                let _ = req.error(err);
            } else {
                let _ = req.ok();
            }
        }

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
