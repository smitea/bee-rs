use crate::{datasource::Status, Promise, Result, ToData, ToType};
use ssh::Session;
use std::{io::Write, path::PathBuf, sync::Arc};

#[datasource]
pub async fn write_file<'a>(
    session: Arc<Session>,
    base_path: String,
    path: String,
    content: String,
    promise: &mut Promise<'a, Status>,
) -> Result<()> {
    let path: PathBuf = path.parse()?;
    smol::block_on(async move { write_context(session, base_path, path, content) })?;
    promise.commit(Status { success: true }).await?;
    Ok(())
}

pub fn write_context(
    session: Arc<Session>,
    base_path: String,
    path: PathBuf,
    content: String,
) -> Result<()> {
    let mut channel = session.scp_new(ssh::Mode::WRITE, base_path)?;
    channel.init()?;
    channel.push_file(path, content.len(), 0o644)?;
    let _ = channel.write(content.as_bytes());

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
            if let Err(err) = write_file(
                session,
                "/tmp".to_string(),
                "test.log".to_owned(),
                "hello world".to_string(),
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
