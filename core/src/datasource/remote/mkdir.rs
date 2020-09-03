use crate::{datasource::Status, Promise, Result, ToData, ToType};
use ssh::Session;
use std::sync::Arc;

#[datasource]
pub async fn mkdir(
    session: Arc<Session>,
    home_dir: String,
    dir: String,
    promise: &mut Promise<'_, Status>,
) -> Result<()> {
    smol::block_on(async move {
        let mut scp = session.scp_new(ssh::Mode::RECURSIVE | ssh::Mode::WRITE, &home_dir)?;
        scp.init()?;
        scp.push_directory(&dir, 0o755)
    })?;

    promise.commit(Status { success: true }).await?;
    Ok(())
}

#[test]
fn test() {
    use crate::*;
    smol::block_on(async {
        let session = super::new_test_sess().unwrap();
        let (req, resp) = crate::new_req(crate::Args::new());
        smol::spawn(async move {
            let mut promise = req.head::<Status>().await.unwrap();
            if let Err(err) = mkdir(
                session,
                "/tmp".to_owned(),
                "bethune".to_owned(),
                &mut promise,
            )
            .await
            {
                let _ = req.error(err);
            } else {
                let _ = req.ok();
            }
        })
        .detach();

        let resp = resp.wait().await.unwrap();
        assert_eq!(&columns![Boolean: "success"], resp.columns());

        let mut index = 0;
        for row in resp {
            let row = row.unwrap();
            let success: bool = row.get(0).unwrap();
            assert!(success);
            index += 1;
        }
        assert!(index > 0);
    });
}
