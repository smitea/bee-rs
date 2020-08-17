use crate::{Columns, Promise, Result, Row, ToData};
use smol::block_on;
use heim::memory::{swap, Swap};

#[derive(Data)]
pub struct SWAPUsage {
    used_bytes: i64,
    total_bytes: i64,
    free_bytes: i64,
}

#[datasource]
pub fn swap_usage(promise: &mut Promise<SWAPUsage>) -> Result<()> {
    let swap: Swap = block_on(swap())?;

    let total = swap.total();
    let free = swap.free();
    promise.commit(SWAPUsage {
        used_bytes: (total - free) as i64,
        total_bytes: total as i64,
        free_bytes: free as i64,
    })?;
    Ok(())
}


#[test]
fn test() {
    use crate::*;
    let (req, resp) = crate::new_req(crate::Args::new(), std::time::Duration::from_secs(2));
    {
        let mut promise = req.head::<SWAPUsage>().unwrap();
        swap_usage(&mut promise).unwrap();
        drop(req);
    }

    let resp = resp.wait().unwrap();
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
}
