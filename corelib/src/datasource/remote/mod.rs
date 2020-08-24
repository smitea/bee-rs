use crate::{code, configure::Configure, register_state, DataSource, Error, Instance, Result};
use ssh::Session;
use parking_lot::RwLock;
use std::sync::Arc;

mod shell;
mod mkdir;
mod read_file;
mod write_file;

const BASE_CODE: i32 = 83 + 83 + 72;

type SSHError = ssh::Error;

impl From<SSHError> for Error {
    fn from(err: SSHError) -> Self {
        let code = err.code;
        let msg = err.msg;
        return Error::other(code!(BASE_CODE, code), msg);
    }
}

pub fn new_session(instance: &Instance) -> Result<Arc<RwLock<Session>>> {
    let protocol = instance.get_connect_mod();

    let host = instance.get_host().ok_or(Error::index_param("host"))?;
    let port: u16 = instance.get_port().ok_or(Error::index_param("port"))?;
    let username = instance
        .get_username()
        .ok_or(Error::index_param("username"))?;
    if username.trim().is_empty() {
        return Err(Error::index_param("username"));
    }

    let connect_timeout: i32 = instance.get_param("connect_timeout").unwrap_or(5);

    let mut sess = Session::new().unwrap();
    sess.set_host(host)?;
    sess.set_port(port as usize)?;
    sess.set_timeout(connect_timeout as usize)?;
    sess.set_username(&username)?;
    sess.connect()?;
    if protocol == "password" {
        let password = instance
            .get_password()
            .ok_or(Error::index_param("password"))?;
        sess.userauth_password(password)?;
    } else if protocol == "pubkey" {
        sess.userauth_agent()?;
    } else {
        return Err(Error::index_param("protocol"));
    }

    return Ok(Arc::new(RwLock::new(sess)));
}

pub fn register_ds<T: Configure>(instance: &Instance, connection: &T) -> Result<()> {
    use crate::register_ds;

    let session = new_session(instance)?;

    let ds = register_ds!(read_file);
    register_state!(ds, session.clone());
    connection.register_source(ds)?;

    let ds = register_ds!(write_file);
    register_state!(ds, session.clone());
    connection.register_source(ds)?;

    let ds = register_ds!(mkdir);
    register_state!(ds, session.clone());
    connection.register_source(ds)?;

    let ds = register_ds!(shell);
    register_state!(ds, session.clone());
    connection.register_source(ds)?;
    Ok(())
}

#[cfg(test)]
#[cfg(feature = "sqlite")]
fn new_test_sess()  -> Result<Arc<RwLock<Session>>>{
    let uri = get_remote_uri();
    let instance: Instance = format!("sqlite:{}",uri).parse()?;
    new_session(&instance)
}

#[cfg(test)]
pub fn get_remote_uri() -> String {
    let port = std::env::var("RUST_SSH_FIXTURE_PORT")
        .map(|s| s.parse().unwrap())
        .unwrap_or(22);
    let user = std::env::var("USER").unwrap();
    return format!("remote:pubkey://{}@127.0.0.1:{}/bee?connect_timeout=5",user,port);
}
