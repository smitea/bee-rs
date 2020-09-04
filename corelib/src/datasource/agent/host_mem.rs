use crate::{Columns, Promise, Result, Row, ToData};
use heim::memory::{memory, Memory};
use async_std::task::block_on;

#[derive(Data)]
pub struct MemoryUsage {
    used_bytes: i64,
    total_bytes: i64,
    free_bytes: i64,
}

#[datasource]
pub fn memory_usage(promise: &mut Promise<MemoryUsage>) -> Result<()> {
    let memory: Memory = block_on(memory())?;

    let total: u64 = memory.total();
    let available: u64 = memory.available();
    promise.commit(MemoryUsage {
        used_bytes: (total - available) as i64,
        total_bytes: total as i64,
        free_bytes: available as i64,
    })?;
    Ok(())
}

#[test]
fn test() {
    use crate::*;
    let (req, resp) = crate::new_req(crate::Args::new(), std::time::Duration::from_secs(2));
    async_std::task::spawn_blocking(move || {
        let mut promise = req.head::<MemoryUsage>().unwrap();
        if let Err(err) = memory_usage(&mut promise) {
            let _ = req.error(err);
        } else {
            let _ = req.ok();
        }
        drop(req);
    });

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
