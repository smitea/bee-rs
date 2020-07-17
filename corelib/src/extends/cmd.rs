use crate::{columns, Columns, DataSource, Error, Instance, Promise, State};
use std::{io::BufRead, io::Cursor, process::Command};

struct CMDSource;

impl DataSource for CMDSource {
    fn name(&self) -> &str {
        "cmd"
    }

    fn args(&self) -> Columns {
        columns![String: "script"]
    }

    fn columns(&self) -> Columns {
        columns![String: "line", Integer: "line_num"]
    }

    fn collect(&self, promise: &mut Promise) -> Result<(), crate::Error> {
        let args = promise.get_args();
        let cmd: String = args.get(0)?;
        return run_cmd(cmd, promise);
    }
}

fn run_cmd(cmd: String, promise: &mut crate::Promise) -> Result<(), crate::Error> {
    let output = Command::new("sh").arg("-c").arg(cmd).output()?;
    if output.status.success() {
        let mut cur = Cursor::new(output.stdout).lines();
        let mut index  = 0;
        while let Some(line) = cur.next() {
            let line = line?;
            promise.commit(State::from(crate::row![line,index]))?;
            index += 1;
        }
        Ok(())
    } else {
        let msg = String::from_utf8(output.stderr).unwrap_or("".to_owned());
        return Err(crate::Error::from(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            format!("exit code: {:?} - {}", output.status, msg),
        )));
    }
}

pub fn new_datasource(_instance: &Instance) -> Result<Box<dyn DataSource>, Error> {
    Ok(Box::new(CMDSource))
}

#[cfg(test)]
mod test {
    use super::new_datasource;
    use crate::{args, new_req, new_req_none, Instance};
    use std::time::Duration;

    #[test]
    fn test_success() {
        let instance: Instance = "cmd://127.0.0.1:49160/bee".parse().unwrap();
        let args = args!["echo hellword"];
        let (req, stat) = new_req_none(args);
        let ds = new_datasource(&instance).unwrap();

        std::thread::spawn(move || {
            let cols = ds.columns();
            let mut promise = req.head(cols).unwrap();
            if let Err(err) = ds.collect(&mut promise) {
                let _ = req.error(err);
            }
        });

        let resp = stat.wait().unwrap();
        let columns = resp.columns();
        assert_eq!(2, columns.len());
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
        let ds = new_datasource(&instance).unwrap();

        std::thread::spawn(move || {
            let cols = ds.columns();
            let mut promise = req.head(cols).unwrap();
            if let Err(err) = ds.collect(&mut promise) {
                let _ = req.error(err);
            }
        });

        let resp = stat.wait().unwrap();
        println!("columns - {:?}", resp.columns());
        for rs in resp {
            let _ = rs.unwrap();
        }
    }

    #[test]
    #[should_panic(expected = "sh: hhscpp: command not found")]
    fn cmd_not_found_invalid() {
        // 262
        let instance: Instance = "cmd://127.0.0.1:49160/bee".parse().unwrap();
        let args = args!["hhscpp qwd12dc"];
        let (req, stat) = new_req(args, Duration::from_secs(2));
        let ds = new_datasource(&instance).unwrap();

        std::thread::spawn(move || {
            let cols = ds.columns();
            let mut promise = req.head(cols).unwrap();
            if let Err(err) = ds.collect(&mut promise) {
                let _ = req.error(err);
            }
        });

        let resp = stat.wait().unwrap();
        println!("columns - {:?}", resp.columns());
        for rs in resp {
            let _ = rs.unwrap();
        }
    }
}
