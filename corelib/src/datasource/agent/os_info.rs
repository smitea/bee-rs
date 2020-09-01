use crate::{Columns, Promise, Result, Row, ToData};
use heim::host::{platform, Platform};
use async_std::task::block_on;

#[derive(Data)]
pub struct OSInfo {
    os_type: String,
    version: String,
    host_name: String,
}

#[datasource]
pub fn os_info(promise: &mut Promise<OSInfo>) -> Result<()> {
    let platform: Platform = block_on(platform())?;

    promise.commit(OSInfo {
        os_type: platform.system().to_string(),
        version: platform.release().to_string(),
        host_name: platform.hostname().to_string(),
    })?;
    Ok(())
}

#[test]
fn test() {
    use crate::*;
    let (req, resp) = crate::new_req(crate::Args::new(), std::time::Duration::from_secs(2));
    {
        let mut promise = req.head::<OSInfo>().unwrap();
        if let Err(err) = os_info(&mut promise) {
            let _ = req.error(err);
        } else {
            let _ = req.ok();
        }
        drop(req);
    }

    let resp = resp.wait().unwrap();
    assert_eq!(
        &columns![String: "os_type",String: "version",String: "host_name"],
        resp.columns()
    );

    let mut index = 0;
    for row in resp {
        let _ = row.unwrap();
        index += 1;
    }
    assert!(index > 0);
}
