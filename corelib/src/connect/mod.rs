use crate::{datasource, funcs, Configure, Error, Instance, Result, Statement};
use std::time::Duration;
use async_trait::async_trait;

#[cfg(feature = "lua")]
mod lua;
#[cfg(feature = "sqlite")]
mod sqlite;

/// 连接器
#[async_trait]
pub trait Connection : Send + Sync{
    /// 创建结果集
    async fn new_statement(&self, script: &str, timeout: Duration) -> Result<Statement>;
}

pub async fn new_connection(url: &str) -> Result<Box<dyn Connection>> {
    let instance: Instance = url.parse()?;

    let mode = instance.get_sess_mode();

    match mode {
        #[cfg(feature = "sqlite")]
        "sqlite" => new_sqlite_connection(&instance).await,
        #[cfg(feature = "lua")]
        "lua" => new_lua_connection(&instance).await,
        _ => return Err(Error::index_param("mode")),
    }
}

#[cfg(feature = "sqlite")]
pub async fn new_sqlite_connection(instance: &Instance) -> Result<Box<dyn Connection>> {
    let connection = sqlite::SqliteSession::new()?;
    register(instance, &connection).await?;
    Ok(Box::new(connection))
}

#[cfg(feature = "lua")]
pub async fn new_lua_connection(instance: &Instance) -> Result<Box<dyn Connection>> {
    let connection = lua::LuaSession::new();
    register(instance, &connection).await?;
    Ok(Box::new(connection))
}

pub async fn register<T: Configure>(instance: &Instance, configure: &T) -> Result<()> {
    // 注册扩展内容
    funcs::register_ds(&instance, configure).await?;
    datasource::register_ds(&instance, configure).await?;

    Ok(())
}
