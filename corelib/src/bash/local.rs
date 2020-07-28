use crate::bash::BashRow;
use crate::{Error, Promise, Result, ToData, ToType};
use std::io::{BufRead, Cursor};
use std::process::Command;

#[datasource]
fn shell(script: String, promise: &mut Promise<BashRow>) -> Result<()> {
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
