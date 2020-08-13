#![feature(seek_convenience)]
#![feature(with_options)]
#![deny(
    unused,
    unused_imports,
    unused_features,
    bare_trait_objects,
    future_incompatible,
    nonstandard_style,
    dead_code,
    deprecated,
)]
#![warn(
    unused_extern_crates,
    unused_import_braces,
    unused_results
)]
#![allow(clippy::missing_safety_doc)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod connect;
mod error;
mod value;

mod args;
mod columns;
mod datatype;
mod instance;
mod request;

mod row;
mod state;
mod statement;
mod register;

mod funcs;
mod datasource;
mod configure;

#[macro_use]
pub mod macros;

pub use funcs::*;
pub use datasource::*;
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

pub use register::Register;
pub use connect::Connection;
pub use datasource::DataSource;
pub use instance::Instance;
pub use request::Promise;
pub use request::Request;
pub use statement::new_req;
pub use statement::new_req_none;
pub use statement::Response;
pub use statement::Statement;
pub use configure::Configure;

pub use datasource::register_ds;

#[macro_use]
extern crate log;
#[macro_use]
extern crate bee_codegen;

/// 创建一个连接，用于执行 SQL
pub fn new_connection(url: &str) -> Result<Box<dyn Connection>> {
    connect::new_connection(url)
}
