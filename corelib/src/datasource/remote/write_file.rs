use crate::{datasource::Status, Promise, Result, ToData, ToType};
use ssh::Session;
use std::{
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[datasource]
pub fn write_file(
    session: Arc<Mutex<Session>>,
    base_path: String,
    path: String,
    content: String,
    promise: &mut Promise<Status>,
) -> Result<()> {
    let path: PathBuf = path.parse()?;
    let mut lock = session.lock()?;
    let mut channel = lock.scp_new(ssh::Mode::WRITE, base_path)?;
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
    {
        let mut promise = req.head::<Status>().unwrap();
        write_file(
            session,
            "/u01/app/oracle".to_string(),
            "test.log".to_owned(),
            "hello world".to_string(),
            &mut promise,
        )
        .unwrap();
        drop(req);
    }

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
