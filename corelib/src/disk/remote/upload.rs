use crate::{disk::Status, Promise, Result, ToData, ToType};
use ssh::Session;
use std::{
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[datasource]
pub fn upload_remote_file(
    session: Arc<Mutex<Session>>,
    path: String,
    content: String,
    promise: &mut Promise<Status>,
) -> Result<()> {
    let path: PathBuf = path.parse()?;
    let mut lock = session.lock()?;
    let mut channel = lock.scp_new(ssh::Mode::WRITE, "/")?;
    channel.init()?;
    channel.push_file(path, content.len(), 0o644)?;
    channel.write(content.as_bytes())?;
    drop(channel);
    promise.commit(Status { success: true })?;
    Ok(())
}
