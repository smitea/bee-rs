use crate::{datasource::Status, Promise, Result,ToType,ToData};
use std::{fs::create_dir_all, path::PathBuf};

#[datasource]
pub fn mkdir(dir: String, promise: &mut Promise<Status>) -> Result<()> {
    let path: PathBuf = dir.parse()?;
    info!("create dir - {}", dir);
    let path = if !path.is_absolute() {
        std::env::current_dir()?.join(path)
    } else {
        path
    };
    create_dir_all(path)?;
    promise.commit(Status { success: true })?;

    Ok(())
}
