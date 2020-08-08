//! 实现 Sqlite 的扩展支持
mod convert;
mod sql_tab;

use crate::Error;
use crate::{new_req, Args, Columns, DataSource, DataType, Request, State, Statement, Value};
use async_std::task;
use convert::INVALIDCOLUMNCOUNT;
use parking_lot::*;
use rusqlite::vtab::eponymous_only_module;
use rusqlite::{Column, Connection, Result, Row, NO_PARAMS};
use sql_tab::SQLTab;
use std::panic::UnwindSafe;
use std::{sync::Arc, time::Duration};

/// Sqlite 连接信息
pub struct SqliteSession {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteSession {
    /// 创建一个 Sqlite 连接 (单线程模式)
    pub fn new() -> Result<Self> {
        Ok(Self {
            connection: Arc::new(Mutex::new(Connection::open_in_memory()?)),
        })
    }
}

impl crate::Configure for SqliteSession {
    fn register_func<F, V: Into<Value>>(
        &self,
        name: &str,
        args: usize,
        func: F,
    ) -> crate::Result<()>
    where
        F: Fn(&Args) -> crate::Result<V> + Send + Sync + UnwindSafe + 'static,
    {
        info!("register function - {}", name);
        let lock = self.connection.lock();

        // 扩展 Sqlite 函数
        lock.create_scalar_function(
            name,
            args as i32,
            rusqlite::functions::FunctionFlags::default(),
            move |context| {
                // 将 Sqlite 函数参数列表转换为 Bee 参数列表
                let mut args_content = Args::new();
                for i in 0..args {
                    args_content.push(context.get::<Value>(i)?);
                }
                // 调用扩展行数
                let value: Value = func(&args_content)?.into();

                Ok(value)
            },
        )?;
        Ok(())
    }

    fn register_source(&self, ds: Box<dyn DataSource>) -> crate::Result<()> {
        let name = ds.name().to_string();
        info!("register datasource - {}", name);
        let aux: Option<Arc<Box<dyn crate::DataSource>>> = Some(Arc::new(ds));
        let lock = self.connection.lock();
        lock.create_module(name.as_str(), eponymous_only_module::<SQLTab>(), aux)?;
        Ok(())
    }
}

impl crate::Connection for SqliteSession {
    fn new_statement(&self, script: &str, timeout: Duration) -> crate::Result<Statement> {
        let (request, response) = new_req(Args::new(), timeout);
        let conn = self.connection.clone();
        let script = script.to_string();

        info!(
            "new_statement for script: {} with timeout = {:?}",
            script, timeout
        );

        // 异步接收该结果，该异步处于协程中
        let _ = task::spawn(async move {
            let req = request;
            let rs = commit_statement(conn, script, &req);
            if let Err(err) = rs {
                let _ = req.error(err);
            }
        });

        Ok(response)
    }
}

/// 提交一个请求，并执行
fn commit_statement(
    db: Arc<Mutex<Connection>>,
    script: String,
    request: &Request,
) -> Result<(), Error> {
    let lock = db.lock();
    let mut s = lock.prepare(script.as_str())?;
    let mut rows = s.query(NO_PARAMS)?;

    // 需要先发送列的结构定义
    // 尝试获取一行数据，才能决定列的类型
    let mut promise = match rows.next()? {
        Some(row) => {
            let new_row = get_row(row)?;
            // 转换列的结构
            let mut cols = Columns::new();
            for i in 0..row.column_count() {
                let name = row.column_name(i)?;
                let value = row.get::<usize, Value>(i)?;

                cols.push(name, DataType::from(value));
            }

            let mut promise = request.new_commit(cols)?;
            promise.commit(State::from(new_row))?;
            promise
        }
        None => {
            // 获取默认的列结构
            let sql_columns = rows.columns().ok_or(Error::invalid(
                INVALIDCOLUMNCOUNT,
                format!("can't find columns"),
            ))?;
            request.new_commit(get_columns(sql_columns))?
        }
    };

    // 循环发送数据
    while let Ok(Some(rs)) = rows.next() {
        promise.commit(State::from(get_row(rs)?))?;
    }
    Ok(())
}

/// 转换数据行为 Bee 格式
fn get_row(rs: &Row) -> Result<crate::Row, Error> {
    let count = rs.column_count();
    let mut row = crate::Row::new();
    for i in 0..count {
        let val = rs.get::<usize, Value>(i)?;
        row.push(val);
    }
    Ok(row)
}

/// 转换数据列为 Bee 格式
fn get_columns(sql_columns: Vec<Column>) -> Columns {
    let mut columns = Columns::new();
    for col in sql_columns {
        let name: &str = col.name();
        let sql_type: Option<&str> = col.decl_type();

        let t = match sql_type {
            Some(t) => {
                let t = t.to_uppercase();
                let t = t.as_str();
                match t {
                    "TEXT" => DataType::String,
                    "INTEGER" => DataType::Integer,
                    "REAL" => DataType::Number,
                    "BLOB" => DataType::Bytes,
                    _ => DataType::Number,
                }
            }
            None => DataType::Nil,
        };
        columns.push(name, t);
    }

    columns
}