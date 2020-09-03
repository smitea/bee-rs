use crate::{Columns, Promise, Result, Row, ToData};
use heim::host::{platform, Platform};

#[derive(Data)]
pub struct OSInfo {
    os_type: String,
    version: String,
    host_name: String,
}

#[datasource]
pub async fn os_info<'a>(promise: &mut Promise<'a, OSInfo>) -> Result<()> {
    let platform: Platform = platform().await?;

    promise
        .commit(OSInfo {
            os_type: platform.system().to_string(),
            version: platform.release().to_string(),
            host_name: platform.hostname().to_string(),
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
            let mut promise = req.head::<OSInfo>().await.unwrap();
            if let Err(err) = os_info(&mut promise).await {
                let _ = req.error(err);
            } else {
                let _ = req.ok();
            }
        }).detach();

        let resp = resp.wait().await.unwrap();
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
    });
}
