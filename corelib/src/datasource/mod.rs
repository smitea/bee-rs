use crate::{Columns, Connection, Instance, Result, Row,Register,Request};

#[cfg(feature = "agent")]
mod agent;
#[cfg(feature = "remote")]
mod remote;

#[derive(Data)]
pub struct FileLine {
    pub line: String,
    pub line_num: u32,
}

#[derive(Data)]
pub struct Status {
    pub success: bool,
}

#[derive(Data)]
pub struct BashRow {
    line: String,
    line_num: u32,
}

pub trait DataSource: Send + Sync {
    fn name(&self) -> &str;
    fn args(&self) -> Columns;
    fn columns(&self) -> Columns;
    fn get_register(&self) -> &Register;
    fn collect(&self,request: &mut Request) -> Result<()>;
}

pub fn register_ds<T: Connection>(instance: &Instance, connection: &T) -> Result<()> {
    let mode = instance.get_connect_mod();

    match mode {
        "agent" => {
            #[cfg(feature = "agent")]
            agent::register_ds(instance, connection)?;
        }
        "remote" => {
            #[cfg(feature = "remote")]
            remote::register_ds(instance, connection)?;
        }
        _ => unimplemented!(),
    }
    Ok(())
}
