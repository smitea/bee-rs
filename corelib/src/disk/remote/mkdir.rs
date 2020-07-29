use crate::{disk::Status, Promise, Result, ToData, ToType};
use ssh::Session;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[datasource]
pub fn remote_mkdir(
    session: Arc<Mutex<Session>>,
    dir: String,
    promise: &mut Promise<Status>,
) -> Result<()> {
    let path: PathBuf = dir.parse()?;
    let mut lock = session.lock()?;
    let mut scp = lock.scp_new(ssh::Mode::RECURSIVE | ssh::Mode::WRITE, "/")?;
    scp.init()?;

    let mut dirs = path.ancestors();
    let mut tmp = Vec::new();
    loop {
        let dir = dirs.next();
        if let Some(path) = dir {
            if Path::new("/") != path {
                tmp.push(path);
            }
        } else {
            break;
        }
    }

    tmp.reverse();
    for dir in tmp {
        scp.push_directory(&dir, 0o755)?;
    }

    promise.commit(Status { success: true })?;
    Ok(())
}
