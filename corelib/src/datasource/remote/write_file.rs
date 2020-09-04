use crate::{datasource::Status, Promise, Result, ToData, ToType};
use ssh::Session;
use std::{io::Write, path::PathBuf, sync::Arc};

#[datasource]
pub fn write_file(
    session: Arc<Session>,
    base_path: String,
    path: String,
    content: String,
    promise: &mut Promise<Status>,
) -> Result<()> {
    let path: PathBuf = path.parse()?;
    let mut channel = session.scp_new(ssh::Mode::WRITE, base_path)?;
    channel.init()?;
    channel.push_file(path, content.len(), 0o644)?;
    let _ = channel.write(content.as_bytes())?;
    drop(channel);
    promise.commit(Status { success: true })?;
    Ok(())
}

#[test]
fn test() {
    use crate::*;
    let session = super::new_test_sess().unwrap();
    let (req, resp) = crate::new_req(crate::Args::new(), std::time::Duration::from_secs(2));
    async_std::task::spawn_blocking(move || {
        let mut promise = req.head::<Status>().unwrap();
        if let Err(err) = write_file(
            session,
            "/tmp".to_string(),
            "test.log".to_owned(),
            "hello world".to_string(),
            &mut promise,
        ) {
            let _ = req.error(err);
        } else {
            let _ = req.ok();
        }
    });

    let resp = resp.wait().unwrap();
    assert_eq!(&columns![Boolean: "success"], resp.columns());

    let mut index = 0;
    for row in resp {
        let row = row.unwrap();
        let success: bool = row.get(0).unwrap();
        assert!(success);
        index += 1;
    }
    assert!(index > 0);
}
