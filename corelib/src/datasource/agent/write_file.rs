use crate::{datasource::Status, Promise, Result, ToData, ToType};
use std::{path::PathBuf};

#[datasource]
pub fn write_file(
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

    std::fs::write(&path, content)?;
    promise.commit(Status { success: true })?;
    Ok(())
}



#[test]
fn test() {
    use crate::*;
    let path = "/tmp/test_file.log".to_string();
    let content = "Hello world".to_owned();
    let (req, resp) = crate::new_req(crate::Args::new(), std::time::Duration::from_secs(2));
    {
        let mut promise = req.head::<Status>().unwrap();
        write_file(path.clone(), content.clone(), &mut promise).unwrap();
        drop(req);
    }

    let resp = resp.wait().unwrap();
    assert_eq!(
        &columns![Boolean: "success"],
        resp.columns()
    );

    let mut index = 0;
    for row in resp {
        let row = row.unwrap();
        let success: bool = row.get(0).unwrap();

        assert!(success);
        index += 1;
    }
    assert_eq!(content.as_bytes(),std::fs::read(&path).unwrap().as_slice());
    std::fs::remove_file(&path).unwrap();
    assert!(index > 0);
}
