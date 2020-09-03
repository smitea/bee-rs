use crate::{datasource::Status, Promise, Result, ToData, ToType};
use std::path::PathBuf;

#[datasource]
pub async fn write_file(
    path: String,
    content: String,
    promise: &mut Promise<'_, Status>,
) -> Result<()> {
    let path: PathBuf = path.parse()?;
    let path = if !path.is_absolute() {
        std::env::current_dir()?.join(path)
    } else {
        path
    };

    std::fs::write(&path, content)?;
    promise.commit(Status { success: true }).await?;
    Ok(())
}

#[test]
fn test() {
    use crate::*;
    smol::block_on(async {
        const PATH: &str = "/tmp/test_file.log";
        const CONTENT: &str = "Hello world";
        let (req, resp) = crate::new_req(crate::Args::new());
        smol::spawn(async move {
            let mut promise = req.head::<Status>().await.unwrap();
            if let Err(err) = write_file(PATH.to_string(), CONTENT.to_string(), &mut promise).await
            {
                let _ = req.error(err);
            } else {
                let _ = req.ok();
            }
        })
        .detach();

        let resp = resp.wait().await.unwrap();
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
    });
}
