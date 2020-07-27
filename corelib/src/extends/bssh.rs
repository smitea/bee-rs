use crate::{columns, row, Columns, DataSource, Error, Instance, Promise, State, CODE};
use parking_lot::*;
use ssh::Session;
use std::{sync::Arc, time::Duration};

const BASE_CODE: i32 = 83 + 83 + 72;
const MARK: &str = "bee";

type SSHError = ssh::Error;

impl From<SSHError> for Error {
    fn from(err: SSHError) -> Self {
        let code = err.code;
        let msg = err.msg;
        return Error::other(CODE!(BASE_CODE, code), msg);
    }
}

pub struct SSHDataSource {
    session: Arc<Mutex<Session>>,
}

impl DataSource for SSHDataSource {
    fn name(&self) -> &str {
        "ssh"
    }

    fn columns(&self) -> Columns {
        columns![String: "line", Integer: "line_num"]
    }

    fn args(&self) -> Columns {
        columns![String: "script", Integer: "timeout"]
    }

    fn collect(&self, promise: &mut Promise) -> Result<(), crate::Error> {
        let args = promise.get_args();
        println!("ssh({:?})",args);
        let script: String = args.get(0)?;
        let timeout: u16 = args.get(1)?;

        run_cmd(self.session.clone(), script, timeout, promise)?;
        Ok(())
    }
}

pub fn run_cmd(
    session: Arc<Mutex<Session>>,
    script: String,
    timeout: u16,
    promise: &mut Promise,
) -> Result<(), crate::Error> {
    let mut lock = session.lock();

    let mut channel = lock.channel_new()?;
    channel.open_session()?;

    // 起始标示
    let mark_start = format!("#{}", MARK);
    // 结束标示
    let mark_end = format!("{}#", MARK);

    let mark_start_cmd = format!("echo '{}'", mark_start);
    let mark_end_cmd = format!("echo '{}'", mark_end);
    let real_script = format!("{};echo '';{};echo '';{};", mark_start_cmd, script, mark_end_cmd);

    println!("run cmd - {}",real_script);
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
            return Err(Error::io_timeout(format!("cmd - [{}] is timeout", script)));
        }
    }

    channel.send_eof()?;
    println!("output - {}",buffer);
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
            promise.commit(State::from(row![line,index as i64]))?;
            index += 1;
        }
    }

    return Ok(());
}



#[cfg(test)]
mod test {
    use super::new_datasource;
    use crate::{args, new_req_none, Instance};

    #[test]
    fn test_success() {
        let instance: Instance =
            "ssh://oracle:admin@127.0.0.1:49160/bee?connect_timeout=5&protocol=user_pwd"
                .parse()
                .unwrap();
        let args = args!["echo hellword", 10];
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
    #[should_panic(expected = "[sleep 3] is timeout")]
    fn cmd_timeout_invalid() {
        // 3335
        let instance: Instance =
            "ssh://oracle:admin@127.0.0.1:49160/bee?connect_timeout=5&protocol=user_pwd"
                .parse()
                .unwrap();
        let args = args!["sleep 3", 2];
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
        println!("columns - {:?}", resp.columns());
        for rs in resp {
            let _ = rs.unwrap();
        }
    }

    #[test]
    #[should_panic(expected = "protocol")]
    fn param_invalid_protocol() {
        // 260
        let instance: Instance = "ssh://oracle:admin@127.0.0.1:12/bee".parse().unwrap();
        let _ = new_datasource(&instance).unwrap();
    }

    #[test]
    #[should_panic(expected = "password")]
    fn param_invalid_password() {
        // 260
        let instance: Instance =
            "ssh://oracle@127.0.0.1:49160/bee?connect_timeout=5&protocol=user_pwd"
                .parse()
                .unwrap();
        let _ = new_datasource(&instance).unwrap();
    }

    #[test]
    #[should_panic(expected = "username")]
    fn param_invalid_username() {
        // 260
        let instance: Instance = "ssh://127.0.0.1:49160/bee?connect_timeout=5&protocol=user_pwd"
            .parse()
            .unwrap();
        let _ = new_datasource(&instance).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Failed to resolve hostname test (nodename nor servname provided, or not known)"
    )]
    fn param_invalid_host() {
        // 192009
        let instance: Instance = "ssh://oracle:admin@test:12/bee?protocol=user_pwd"
            .parse()
            .unwrap();
        let _ = new_datasource(&instance).unwrap();
    }

    #[test]
    #[should_panic(expected = "Connection refused")]
    fn connect_timeout_invalid() {
        // 192009
        let instance: Instance =
            "ssh://oracle:admin@127.0.0.1:12/bee?connect_timeout=5&protocol=user_pwd"
                .parse()
                .unwrap();
        let _ = new_datasource(&instance).unwrap();
    }

    #[test]
    #[should_panic(expected = "Failed to connect: No route to host")]
    fn host_invalid() {
        // 192009
        let instance: Instance =
            "ssh://oracle:admin@127:12/bee?connect_timeout=5&protocol=user_pwd"
                .parse()
                .unwrap();
        let _ = new_datasource(&instance).unwrap();
    }

    #[test]
    #[should_panic(expected = " Authentication that can continue: ")]
    fn pub_key_invalid() {
        // 126473
        let instance: Instance =
            "ssh://oracle:admin@127.0.0.1:49160/bee?connect_timeout=5&protocol=pub_key&public_key=~/.ssh/scrape_ssh.pub"
                .parse()
                .unwrap();
        let _ = new_datasource(&instance).unwrap();
    }

    #[test]
    #[should_panic(expected = "Authentication that can continue:")]
    fn password_invalid() {
        // 126473
        let instance: Instance =
            "ssh://oracle:none@127.0.0.1:49160/bee?connect_timeout=5&protocol=user_pwd"
                .parse()
                .unwrap();
        let _ = new_datasource(&instance).unwrap();
    }
}
