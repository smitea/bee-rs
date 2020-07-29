use crate::{datasource::Status, Error, Promise, Result, ToData, ToType};
use std::{fs::File, io::ErrorKind, io::Write, path::PathBuf};

#[datasource]
pub fn upload_agent_file(
    path: String,
    content: String,
    promise: &mut Promise<Status>,
) -> Result<()> {
    let path: PathBuf = path.parse()?;
    let path = if !path.is_absolute() {
        std::env::current_dir()?.join(path)
    } else {
        path
    };

    let mut file = if path.exists() {
        match File::with_options().append(false).write(true).open(&path) {
            Ok(file) => file,
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    File::create(&path)?
                } else {
                    return Err(Error::from(err));
                }
            }
        }
    } else {
        File::create(&path)?
    };

    file.write(content.as_bytes())?;
    promise.commit(Status { success: true })?;
    Ok(())
}
