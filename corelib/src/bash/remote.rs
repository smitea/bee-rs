use crate::bash::BashRow;
use crate::{code, Connection, Error, Instance, Promise, Result};
use ssh::Session;
use std::time::Duration;
use std::{
    str::Utf8Error,
    sync::{Arc, Mutex, MutexGuard},
};

const BASE_CODE: i32 = 83 + 83 + 72;
const MARK: &str = "bee";

type SSHError = ssh::Error;

impl From<SSHError> for Error {
    fn from(err: SSHError) -> Self {
        let code = err.code;
        let msg = err.msg;
        return Error::other(code!(BASE_CODE, code), msg);
    }
}

fn new_shell(
    mut session: Arc<Mutex<Session>>,
    script: &str,
    timeout: Duration,
    promise: &mut Promise<BashRow>,
) -> Result<()> {
    let mut lock = session.lock()?;
    let mut channel = lock.channel_new()?;
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
        let size = stdout.read_timeout(&mut buf, timeout)?;

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
                "cmd - [{}] is timeout",
                script
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

fn new_session(instance: &Instance) -> Result<Session> {
    let protocol: String = instance.get_param("protocol")?;

    let host = instance.get_host().ok_or(Error::index_param("host"))?;
    let port: u16 = instance.get_port().ok_or(Error::index_param("port"))?;
    let username = instance.get_username();
    if username.trim().is_empty() {
        return Err(Error::index_param("username"));
    }

    let connect_timeout: i32 = instance.get_param("connect_timeout").unwrap_or(5);

    let mut sess = Session::new().unwrap();
    sess.set_host(host)?;
    sess.set_port(port as usize)?;
    sess.set_timeout(connect_timeout as usize)?;
    sess.set_username(username)?;
    sess.connect()?;
    if protocol == "user_pwd" {
        let password = instance
            .get_password()
            .ok_or(Error::index_param("password"))?;
        sess.userauth_password(password)?;
    } else if protocol == "pub_key" {
        let public_key: String = instance.get_param("public_key")?;
        sess.userauth_publickey_auto(Option::Some(public_key.as_str()))?;
    } else {
        return Err(Error::index_param("protocol"));
    }

    return Ok(sess);
}

pub fn register_state<T: Connection>(instance: &Instance, connection: T) -> Result<()> {
    let session = new_session(instance)?;
    connection.register_state::<Arc<Mutex<Session>>>(Arc::new(Mutex::new(session)))?;
    Ok(())
}
