mod connect;
mod error;
mod value;

#[macro_use]
mod args;
#[macro_use]
mod columns;
mod datatype;
mod instance;
mod request;
#[macro_use]
mod row;
mod datasource;
mod state;
mod statement;

#[cfg(feature = "agent")]
mod agent;
mod bash;
mod common;
mod disk;
mod sqlite;

pub use args::Args;
pub use columns::Columns;
pub use error::Error;
pub use error::Result;
pub use value::Value;

pub use crate::state::State;
pub use crate::state::ToData;
pub use datatype::DataType;
pub use datatype::ToType;
pub use row::Row;

pub use connect::Connection;
pub use datasource::DataSource;
pub use instance::Instance;
pub use request::Promise;
pub use request::Request;
pub use statement::new_req;
pub use statement::new_req_none;
pub use statement::Response;
pub use statement::Statement;

#[macro_use]
extern crate log;
#[macro_use]
extern crate bee_codegen;

pub fn new_connection(url: &str) -> Result<sqlite::SqliteSession> {
    let instance: Instance = url.parse()?;
    let connect = sqlite::SqliteSession::new()?;
    common::register_ds(&instance, &connect)?;
    disk::register_ds(&instance, &connect)?;
    bash::register_ds(&instance, &connect)?;

    #[cfg(feature = "agent")]
    agent::register_ds(&instance, &connect)?;
    Ok(connect)
}
