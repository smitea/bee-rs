use crate::{datasource::Status, Promise, Result, ToData, ToType};
use std::path::PathBuf;

#[datasource]
pub fn write_file(path: String, content: String, promise: &mut Promise<Status>) -> Result<()> {
    let path: PathBuf = path.parse()?;
    let path = if !path.is_absolute() {
        std::env::current_dir()?.join(path)
    } else {
        path
    };

    std::fs::write(&path, content)?;
    promise.commit(Status { success: true })?;
    Ok(())
}

#[test]
fn test() {
    use crate::*;
    const PATH: &str = "/tmp/test_file.log";
    const CONTENT: &str = "Hello world";
    let (req, resp) = crate::new_req(crate::Args::new(), std::time::Duration::from_secs(2));
    async_std::task::spawn_blocking(move || {
        let mut promise = req.head::<Status>().unwrap();
        if let Err(err) = write_file(PATH.to_string(), CONTENT.to_string(), &mut promise) {
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
    assert_eq!(CONTENT.as_bytes(), std::fs::read(&PATH).unwrap().as_slice());
    std::fs::remove_file(&PATH).unwrap();
    assert!(index > 0);
}
