use crate::bash::{Bash, BashRow};
use bee_core::{Promise, Error,Result, State, Instance, Driver, DataSource};
use std::process::Command;
use std::io::{Cursor, BufRead};
use std::time::Duration;

pub struct LocalBash {
    instance: Instance,
}

impl Bash for LocalBash {
    fn run_cmd(&self, script: &str, _timeout: Duration, promise: &mut Promise<BashRow>) -> Result<()> {
        let output = Command::new("sh").arg("-c").arg(script).output()?;
        if output.status.success() {
            let mut cur = Cursor::new(output.stdout).lines();
            let mut index = 0;
            while let Some(line) = cur.next() {
                let line = line?;
                promise.commit(BashRow::new(&line, index))?;
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
}

pub fn new_local_bash(instance: Instance) -> Result<Box<dyn Bash>> {
    Ok(Box::new(LocalBash {
        instance,
    }))
}

