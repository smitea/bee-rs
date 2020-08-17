use crate::{BashRow, Promise, Result, ToData, ToType};

#[datasource]
fn shell(output: String, _timeout: u32, promise: &mut Promise<BashRow>) -> Result<()> {
    let lines = output.lines();

    let mut index = 0;
    for line in lines {
        promise.commit(BashRow {
            line: line.to_string(),
            line_num: index,
        })?;
        index += 1;
    }
    Ok(())
}

#[test]
fn test(){
    use crate::*;
    let (req, resp) = crate::new_req(crate::Args::new(), std::time::Duration::from_secs(2));
    {
        let mut promise = req.head::<BashRow>().unwrap();
        shell(r#"
            echo Hello world
            > Hello world
        "#.to_owned(),10,&mut promise).unwrap();
        drop(req);
    }

    let resp = resp.wait().unwrap();
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
}