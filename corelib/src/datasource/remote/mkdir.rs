use crate::{datasource::Status, Error, Promise, Result, ToData, ToType};
use parking_lot::RwLock;
use ssh::Session;
use std::{sync::Arc, time::Duration};

#[datasource]
pub fn mkdir(
    session: Arc<RwLock<Session>>,
    home_dir: String,
    dir: String,
    timeout: u32,
    promise: &mut Promise<Status>,
) -> Result<()> {
    let mut session = session
        .try_write_for(Duration::from_secs(timeout as u64))
        .ok_or(Error::lock_faild("lock timeout at 'mkdir'"))?;
    let mut scp = session.scp_new(ssh::Mode::RECURSIVE | ssh::Mode::WRITE, &home_dir)?;
    scp.init()?;
    scp.push_directory(&dir, 0o755)?;

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
        if let Err(err) = mkdir(
            session,
            "/tmp".to_owned(),
            "bethune".to_owned(),
            10,
            &mut promise,
        ) {
            let _ = req.error(err);
        } else {
            let _ = req.ok();
        }
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
