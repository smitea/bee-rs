use crate::{DataSource, Error, Instance, State};
use parking_lot::*;
use std::{io::BufRead, io::Cursor, process::Command, sync::Arc, time::Duration};

struct CMDSource;

impl DataSource for CMDSource {
    fn name(&self) -> &str {
        "cmd"
    }

    fn collect(&self, request: &crate::Request) -> Result<(), crate::Error> {
        let mut promise = request.head(crate::columns![String: "line"])?;
        let args = request.get_args();
        let cmd: String = args.get(0)?;
        return run_cmd(cmd, &mut promise);
    }
}

fn run_cmd(cmd: String, promise: &mut crate::Promise) -> Result<(), crate::Error> {
    let output = Command::new("sh").arg("-c").arg(cmd).output()?;
    if output.status.success() {
        let mut cur = Cursor::new(output.stdout).lines();
        while let Some(line) = cur.next() {
            let line = line?;
            promise.commit(State::from(crate::row![line]))?;
        }
        Ok(())
    } else {
        let msg = String::from_utf8(output.stderr).unwrap_or("".to_owned());
        return Err(crate::Error::from(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("exit code: {:?} - {}", output.status, msg),
        )));
    }
}

pub fn new_session(_instance: &Instance) -> Result<Box<dyn DataSource>, Error> {
    Ok(Box::new(CMDSource))
}

#[cfg(test)]
mod test {
    use super::new_session;
    use crate::{args, new_req, new_req_none, Instance};
    use std::time::Duration;

    #[test]
    fn test_success() {
        let instance: Instance = "cmd://127.0.0.1:49160/bee".parse().unwrap();
        let args = args!["echo hellword"];
        let (req, stat) = new_req_none(args);
        let ds = new_session(&instance).unwrap();

        std::thread::spawn(move || {
            if let Err(err) = ds.collect(&req) {
                let _ = req.error(err);
            }
        });

        let resp = stat.wait().unwrap();
        let columns = resp.columns();
        assert_eq!(1, columns.len());
        println!("columns - {:?}", columns);

        let mut count = 0;
        for rs in resp {
            let row = rs.unwrap();
            println!("row - {:?}", row);
            count += 1;
        }
        assert_eq!(1, count);
    }

    #[test]
    #[should_panic(expected = "timed out waiting on channel")]
    fn cmd_timeout_invalid() {
        // 262
        let instance: Instance = "cmd://127.0.0.1:49160/bee".parse().unwrap();
        let args = args!["sleep 3"];
        let (req, stat) = new_req(args, Duration::from_secs(2));
        let ds = new_session(&instance).unwrap();

        std::thread::spawn(move || {
            if let Err(err) = ds.collect(&req) {
                println!("has a err - {:?}", err);
                let _ = req.error(err);
            }
        });

        let resp = stat.wait().unwrap();
        println!("columns - {:?}", resp.columns());
        for rs in resp {
            println!("rs - {:?}", rs);
            let _ = rs.unwrap();
        }
    }
}
