use crate::datasource::BashRow;
use crate::{Promise, Result, ToData, ToType};
use std::io::Read;
use std::process::Command;
use std::time::Duration;
use timeout_readwrite::TimeoutReadExt;

#[datasource]
fn shell(script: String, timeout: u32, promise: &mut Promise<BashRow>) -> Result<()> {
    let mut cmd = if cfg!(target_os = "windows") {
        let mut cmd = Command::new("cmd");
        cmd.args(&["/C", &script]);
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(&script);
        cmd
    };

    let child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    let stdout = child.stdout.ok_or(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "lose stdout",
    ))?;

    let mut data = String::new();
    let _ = stdout
        .with_timeout(Duration::from_secs(timeout as u64))
        .read_to_string(&mut data)?;

    let mut cur = data.lines();
    let mut index = 0;
    while let Some(line) = cur.next() {
        promise.commit(BashRow {
            line: line.to_owned(),
            line_num: index,
        })?;
        index += 1;
    }
    Ok(())
}

#[test]
fn test() {
    use crate::*;
    let (req, resp) = crate::new_req(crate::Args::new(), std::time::Duration::from_secs(2));
    {
        let mut promise = req.head::<BashRow>().unwrap();
        if let Err(err) = shell("echo 'Hello world'".to_string(), 2, &mut promise) {
            let _ = req.error(err);
        } else {
            let _ = req.ok();
        }
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

#[test]
#[should_panic(expected = "timed out")]
fn test_shell_timeout() {
    use crate::*;
    let (req, resp) = crate::new_req(crate::Args::new(), std::time::Duration::from_secs(2));
    {
        println!("new shell");
        let mut promise = req.head::<BashRow>().unwrap();
        if let Err(err) = shell("sleep 10;echo 'Hello world'".to_string(), 2, &mut promise) {
            let _ = req.error(err);
        } else {
            let _ = req.ok();
        }
    }

    let resp = resp.wait().unwrap();
    assert_eq!(
        &columns![String: "line",Integer: "line_num"],
        resp.columns()
    );
    for row in resp {
        let _ = row.unwrap();
    }
}
