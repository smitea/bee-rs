use crate::{Columns, Promise, Result, Row, ToData};
use std::time::Duration;

#[derive(Data, PartialEq)]
pub struct CPUUsage {
    idle: f64,
    user: f64,
    system: f64,
    iowait: f64,
}

#[datasource]
pub async fn cpu_usage<'a>(promise: &mut Promise<'a, CPUUsage>) -> Result<()> {
    let cpu_usage = heim::cpu::usage(Duration::from_secs(1)).await?;

    promise
        .commit(CPUUsage {
            idle: cpu_usage.idle(),
            user: cpu_usage.user(),
            system: cpu_usage.system(),
            iowait: 0.0,
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
            let mut promise = req.head::<CPUUsage>().await.unwrap();
            if let Err(err) = cpu_usage(&mut promise).await {
                let _ = req.error(err);
            } else {
                let _ = req.ok();
            }
        }).detach();

        let resp = resp.wait().await.unwrap();
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
    });
}
