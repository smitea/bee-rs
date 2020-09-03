use crate::{Columns, Configure, Instance, Register, Request, Result, Row};

#[cfg(feature = "agent")]
mod agent;
mod debug;
#[cfg(unix)]
#[cfg(feature = "remote")]
mod remote;

#[derive(Data)]
pub struct Status {
    pub success: bool,
}

#[derive(Data)]
pub struct BashRow {
    line: String,
    line_num: u32,
}

#[async_trait]
pub trait DataSource: Send + Sync {
    fn name(&self) -> &str;
    fn args(&self) -> Columns;
    fn columns(&self) -> Columns;
    fn get_register(&self) -> &Register;
    async fn collect(&self, request: &mut Request) -> crate::Result<()>;
}

/// 注册数据源
pub async fn register_ds<T: Configure>(instance: &Instance, connection: &mut T) -> Result<()> {
    let mode = instance.get_ds_mode();
    let ex = std::sync::Arc::new(smol::Executor::new());
    match mode {
        #[cfg(feature = "agent")]
        "agent" => {
            agent::register_ds(instance, ex, connection).await?;
        }
        #[cfg(feature = "remote")]
        #[cfg(unix)]
        "remote" => {
            remote::register_ds(instance, ex, connection).await?;
        }
        "debug" => {
            debug::register_ds(instance, ex, connection).await?;
        }
        _ => unimplemented!(),
    }
    Ok(())
}
