use crate::{datasource::Status, Promise, Result, ToData, ToType};
use ssh::Session;
use std::sync::{Arc, Mutex};

#[datasource]
pub fn mkdir(
    session: Arc<Mutex<Session>>,
    home_dir: String,
    dir: String,
    promise: &mut Promise<Status>,
) -> Result<()> {
    let mut lock = session.lock()?;
    let mut scp = lock.scp_new(ssh::Mode::RECURSIVE | ssh::Mode::WRITE, &home_dir)?;
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
        mkdir(
            session,
            "/home".to_owned(),
            "bethune".to_owned(),
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
