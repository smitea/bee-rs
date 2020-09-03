use crate::{BashRow, Promise, Result, ToData, ToType};

#[datasource]
pub async fn shell(
    output: String,
    _timeout: u32,
    promise: &mut Promise<'_, BashRow>,
) -> Result<()> {
    let lines = output.lines();

    let mut index = 0;
    for line in lines {
        promise
            .commit(BashRow {
                line: line.to_string(),
                line_num: index,
            })
            .await?;
        index += 1;
    }
    Ok(())
}

#[test]
fn test() {
    use crate::*;
    smol::block_on(async {
        let (req, resp) = crate::new_req(crate::Args::new());
        smol::spawn(async move {
            let mut promise = req.head::<BashRow>().await.unwrap();
            if let Err(err) = shell(
                r#"
            echo Hello world
            > Hello world
            "#
                .to_owned(),
                10,
                &mut promise,
            )
            .await
            {
                let _ = req.error(err);
            } else {
                let _ = req.ok();
            }
            drop(req);
        })
        .detach();

        let resp = resp.wait().await.unwrap();
        assert_eq!(
            &columns![String: "line",Integer: "line_num"],
            resp.columns()
        );

        let mut index = 0;
        for row in resp {
            let _ = row.unwrap();
            index += 1;
        }
        assert!(index > 0);
    });
}
