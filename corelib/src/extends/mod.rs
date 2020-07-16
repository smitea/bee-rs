use crate::{Error, Instance, Session};

#[cfg(feature = "agent")]
pub mod agent;
#[cfg(feature = "bssh")]
pub mod bssh;
#[cfg(feature = "cmd")]
pub mod cmd;
#[cfg(feature = "sql")]
pub mod sql;

#[cfg(feature = "sql")]
pub fn new_session(url: &str) -> Result<Box<dyn Session>, Error> {
    let mut sess = sql::new_session()?;
    init_datasource(url, &mut sess)?;
    return Ok(Box::new(sess));
}

pub fn init_datasource<'a>(url: &str, sess: &'a mut dyn Session) -> Result<(), Error> {
    let instance: Instance = url.parse()?;

    #[cfg(feature = "bssh")]
    init_ssh_datasource(&instance, sess)?;
    #[cfg(feature = "cmd")]
    init_cmd_datasource(&instance, sess)?;
    #[cfg(feature = "agent")]
    init_agent_datasource(&instance, sess)?;

    Ok(())
}

#[cfg(feature = "bssh")]
fn init_ssh_datasource(instance: &Instance, sess: &mut dyn Session) -> Result<(), Error> {
    let ssh_ds = bssh::new_datasource(instance)?;
    sess.register_source(ssh_ds)?;
    Ok(())
}

#[cfg(feature = "cmd")]
fn init_cmd_datasource(instance: &Instance, sess: &mut dyn Session) -> Result<(), Error> {
    let cmd = cmd::new_datasource(instance)?;
    sess.register_source(cmd)?;
    Ok(())
}

#[cfg(feature = "agent")]
fn init_agent_datasource(_: &Instance, sess: &mut dyn Session) -> Result<(), Error> {
    agent::init_datasource(sess)?;
    Ok(())
}
