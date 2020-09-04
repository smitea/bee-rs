use crate::{datasource, funcs, Configure, Error, Instance, Result, Statement};
use std::time::Duration;

#[cfg(feature = "lua")]
mod lua;
#[cfg(feature = "sqlite")]
mod sqlite;

/// 连接器
pub trait Connection: Send + Sync {
    /// 创建结果集
    fn new_statement(&self, script: &str, timeout: Duration) -> Result<Statement>;
}

pub fn new_connection(url: &str) -> Result<Box<dyn Connection>> {
    let instance: Instance = url.parse()?;

    let mode = instance.get_sess_mode();

    match mode {
        #[cfg(feature = "sqlite")]
        "sqlite" => new_sqlite_connection(&instance),
        #[cfg(feature = "lua")]
        "lua" => new_lua_connection(&instance),
        _ => return Err(Error::index_param("mode")),
    }
}

#[cfg(feature = "sqlite")]
pub fn new_sqlite_connection(instance: &Instance) -> Result<Box<dyn Connection>> {
    let connection = sqlite::SqliteSession::new()?;
    register(instance, &connection)?;
    Ok(Box::new(connection))
}

#[cfg(feature = "lua")]
pub fn new_lua_connection(instance: &Instance) -> Result<Box<dyn Connection>> {
    let connection = lua::LuaSession::new();
    register(instance, &connection)?;
    Ok(Box::new(connection))
}

pub fn register<T: Configure>(instance: &Instance, configure: &T) -> Result<()> {
    // 注册扩展内容
    funcs::register_ds(&instance, configure)?;
    datasource::register_ds(&instance, configure)?;

    Ok(())
}
