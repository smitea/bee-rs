use crate::{datasource::Status, Promise, Result, ToData, ToType};
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

#[test]
fn test() {
    use crate::*;
    let path = "/tmp/test/bethune".to_owned();
    let (req, resp) = crate::new_req(crate::Args::new(), std::time::Duration::from_secs(2));
    {
        let mut promise = req.head::<Status>().unwrap();
        if let Err(err) = mkdir(path.clone(), &mut promise) {
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
        let _ = row.unwrap();
        index += 1;
    }
    assert!(index > 0);

    let dir = std::fs::read_dir(&path).unwrap();
    assert_eq!(0, dir.count());
    std::fs::remove_dir(&path).unwrap();
}
