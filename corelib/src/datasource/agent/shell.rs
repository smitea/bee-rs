use crate::datasource::BashRow;
use crate::{Error, Promise, Result, ToData, ToType};
use std::io::{BufRead, Cursor};
use std::process::Command;

#[datasource]
fn shell(script: String, _timeout: u32, promise: &mut Promise<BashRow>) -> Result<()> {
    let output = Command::new("sh").arg("-c").arg(script).output()?;
    if output.status.success() {
        let mut cur = Cursor::new(output.stdout).lines();
        let mut index = 0;
        while let Some(line) = cur.next() {
            let line = line?;
            promise.commit(BashRow {
                line,
                line_num: index,
            })?;
            index += 1;
        }
        Ok(())
    } else {
        let msg = String::from_utf8(output.stderr).unwrap_or("".to_owned());
        return Err(Error::from(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            format!("exit code: {:?} - {}", output.status, msg),
        )));
    }
}

#[test]
fn test() {
    use crate::*;
    let (req, resp) = crate::new_req(crate::Args::new(), std::time::Duration::from_secs(2));
    {
        let mut promise = req.head::<BashRow>().unwrap();
        shell("echo 'Hello world'".to_string(), 2, &mut promise).unwrap();
        drop(req);
    }

    let resp = resp.wait().unwrap();
    assert_eq!(
        &columns![String: "line",Integer: "line_num"],
        resp.columns()
    );

    let mut index = 0;
    for row in resp {
        let row = row.unwrap();
        let line: String = row.get(0).unwrap();
        let line_num: i64 = row.get(1).unwrap();

        assert_eq!("Hello world".to_owned(), line);
        assert_eq!(0, line_num);
        index += 1;
    }
    assert!(index > 0);
}
