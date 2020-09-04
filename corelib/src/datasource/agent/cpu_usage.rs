use crate::{Columns, Promise, Result, Row, ToData};
use async_std::task::block_on;
use std::time::Duration;

#[derive(Data, PartialEq)]
pub struct CPUUsage {
    idle: f64,
    user: f64,
    system: f64,
    iowait: f64,
}

#[datasource]
pub fn cpu_usage(promise: &mut Promise<CPUUsage>) -> Result<()> {
    let cpu_usage = block_on(heim::cpu::usage(Duration::from_secs(1)))?;

    promise.commit(CPUUsage {
        idle: cpu_usage.idle(),
        user: cpu_usage.user(),
        system: cpu_usage.system(),
        iowait: 0.0,
    })?;
    Ok(())
}

#[test]
fn test() {
    use crate::*;
    let (req, resp) = crate::new_req(crate::Args::new(), Duration::from_secs(2));
    async_std::task::spawn_blocking(move || {
        let mut promise = req.head::<CPUUsage>().unwrap();
        if let Err(err) = cpu_usage(&mut promise) {
            let _ = req.error(err);
        } else {
            let _ = req.ok();
        }
    });

    let resp = resp.wait().unwrap();
    assert_eq!(
        &columns![Number: "idle",Number: "user", Number: "system", Number: "iowait"],
        resp.columns()
    );

    let mut index = 0;
    for row in resp {
        let _ = row.unwrap();
        index += 1;
    }
    assert!(index > 0);
}
