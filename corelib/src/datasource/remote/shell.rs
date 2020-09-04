use crate::datasource::BashRow;
use crate::{Error, Promise, Result, ToData, ToType};
use ssh::Session;
use std::sync::Arc;
use std::time::Duration;

#[datasource]
fn shell(
    session: Arc<Session>,
    script: String,
    timeout: u32,
    promise: &mut Promise<BashRow>,
) -> Result<()> {
    info!("ssh [{}] with timeout = {}s", script, timeout);
    let mut channel = session.channel_new()?;
    channel.open_session()?;

    let mark = std::thread::current().id();
    // 起始标示
    let mark_start = format!("#{:?}", mark);
    // 结束标示
    let mark_end = format!("{:?}#", mark);

    let mark_start_cmd = format!("echo '{}'", mark_start);
    let mark_end_cmd = format!("echo '{}'", mark_end);
    let real_script = format!(
        "{};echo '';{};echo '';{};",
        mark_start_cmd, script, mark_end_cmd
    );

    channel.request_exec(real_script.as_bytes())?;

    let mut stdout = channel.stdout();

    let mut buffer: String = String::new();

    loop {
        let mut buf = [0u8; 1024];
        let size = stdout.read_timeout(&mut buf, Duration::from_secs(timeout as u64))?;

        if size > 0 {
            let slice = &buf[0..size];
            let rs = std::str::from_utf8(slice)?;

            buffer += rs;

            if buffer.contains(&format!("\n{}", mark_end)) {
                break;
            }
        } else {
            channel.send_eof()?;
            return Err(Error::io_timeout(format!(
                "cmd - [{}] is timeout in {} s",
                script, timeout
            )));
        }
    }

    channel.send_eof()?;
    decode_output(mark_start, mark_end, buffer, promise)?;

    return Ok(());
}

fn decode_output(
    mark_start: String,
    mark_end: String,
    buffer: String,
    promise: &mut Promise<BashRow>,
) -> crate::Result<()> {
    let lines = buffer.lines();
    let mut has_start = false;

    let mut index = 0;
    for line in lines {
        // 匹配起始行
        if line.trim() == mark_start {
            has_start = true;
            continue;
        }

        // 匹配结束行，结束行需要前面两个条件满足后
        if line.trim() == mark_end && has_start {
            return Ok(());
        }

        if has_start && !line.trim().is_empty() {
            promise.commit(BashRow {
                line: line.to_string(),
                line_num: index,
            })?;
            index += 1;
        }
    }
    return Ok(());
}

#[test]
fn test() {
    use crate::*;
    let session = super::new_test_sess().unwrap();
    let (req, resp) = crate::new_req(crate::Args::new(), std::time::Duration::from_secs(2));
    async_std::task::spawn_blocking(move || {
        let mut promise = req.head::<BashRow>().unwrap();
        if let Err(err) = shell(session, "echo 'Hello world'".to_owned(), 2, &mut promise) {
            let _ = req.error(err);
        } else {
            let _ = req.ok();
        }
    });

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
#[should_panic(expected = "timeout")]
fn test_timeout() {
    use crate::*;
    let session = super::new_test_sess().unwrap();
    let (req, resp) = crate::new_req(crate::Args::new(), std::time::Duration::from_secs(2));
    {
        let mut promise = req.head::<BashRow>().unwrap();
        if let Err(err) = shell(
            session,
            "sleep(5),echo 'Hello world'".to_owned(),
            2,
            &mut promise,
        ) {
            let _ = req.error(err);
        } else {
            let _ = req.ok();
        }
        drop(req);
    }

    let resp = resp.wait().unwrap();
    for row in resp {
        let _ = row.unwrap();
    }
}
