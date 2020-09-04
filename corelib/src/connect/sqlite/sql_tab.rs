use crate::Value;
use crate::{new_req_none, Args, Columns, DataSource, DataType, Response, Row};
use rusqlite::ffi;
use rusqlite::vtab::{
    Context, CreateVTab, IndexConstraintOp, IndexInfo, VTab, VTabConnection, VTabCursor, Values,
};
use rusqlite::{types::FromSql, Error, Result};
use std::marker::PhantomData;
use std::{os::raw::c_int, sync::Arc};

/// 统一的 Sqlite 虚拟表定义
#[repr(C)]
pub struct SQLTab {
    /// Base class. Must be first
    base: ffi::sqlite3_vtab,
    ds: Arc<Box<dyn DataSource>>,
    cols: Columns,
    params: Columns,
    args: Option<Args>,
    offset_first_row: usize,
}

/// 将 Bee 的列类型转换为 Sqlite 类型
fn to_sqlite_type<'a>(d_type: DataType) -> &'a str {
    match d_type {
        crate::DataType::String => "TEXT",
        crate::DataType::Integer => "INTEGER",
        crate::DataType::Number => "REAL",
        crate::DataType::Boolean => "INTEGER",
        crate::DataType::Bytes => "BLOB",
        crate::DataType::Nil => "NULL",
    }
}

/// 将 Bee 的列结构和参数列表转换为 Sqlite 虚拟表定义 SQL
fn to_dml_sql(name: &str, args: &Columns, columns: &Columns) -> String {
    let mut sql = format!("CREATE TABLE {}(", name);

    // 生成列名的定义 SQL
    for (index, col) in columns.iter().enumerate() {
        sql.push_str(&col.0);
        sql.push_str(" ");

        sql.push_str(to_sqlite_type(col.1));
        if index < columns.len() - 1 {
            sql.push_str(", ");
        }
    }

    // 如果没有列结构定义和参数列表则不拼接中间的分号
    if columns.len() > 0 && args.len() > 0 {
        sql.push(',');
    }

    // 生成参数名的定义 SQL
    for (index, col) in args.iter().enumerate() {
        sql.push_str(&col.0);
        sql.push_str(" HIDDEN ");

        sql.push_str(to_sqlite_type(col.1));
        if index < args.len() - 1 {
            sql.push_str(", ");
        }
    }

    sql.push_str(");");

    sql
}

unsafe impl<'vtab> VTab<'vtab> for SQLTab {
    type Aux = Arc<Box<dyn DataSource>>;
    type Cursor = SQLTabCursor<'vtab>;

    fn connect(
        _: &mut VTabConnection,
        aux: Option<&Self::Aux>,
        _args: &[&[u8]],
    ) -> Result<(String, Self)> {
        // 获取给定的数据源
        let ds = match aux {
            Some(ds) => ds,
            None => return Err(Error::ModuleError("No datasource".to_owned())),
        };

        // 生成虚拟表创建 SQL
        let columns = ds.columns();
        let args = ds.args();
        let name = ds.name();
        let sql = to_dml_sql(name, &args, &columns);
        debug!("DML - {:?}", sql);

        // 创建虚拟表实例
        let vtab = SQLTab {
            base: ffi::sqlite3_vtab::default(),
            ds: ds.clone(),
            offset_first_row: 0,
            cols: columns,
            params: args,
            args: None,
        };

        Ok((sql, vtab))
    }

    fn best_index(&self, info: &mut IndexInfo) -> Result<()> {
        // 记录索引号的 bit 位，格式为 `0000 1100` => 表示第二列和第三列为可过滤的参数
        let mut params = vec![];
        for (i, constraint) in info.constraints().enumerate() {
            // 必须为 HIDDEN 属性的列才能被检索
            if !constraint.is_usable() {
                continue;
            }

            // 只允许 `=` 操作才能被列入索引
            if constraint.operator() == IndexConstraintOp::SQLITE_INDEX_CONSTRAINT_EQ {
                params.push((i, constraint.column()));
            }
        }

        // 设置索引信息
        for index in params {
            let mut constraint_usage = info.constraint_usage(index.0);
            constraint_usage.set_argv_index(index.1 - (self.cols.len() as i32) + 1);
            constraint_usage.set_omit(true);
        }
        Ok(())
    }

    fn open(&self) -> Result<SQLTabCursor<'_>> {
        // 创建游标
        Ok(SQLTabCursor::new(self.ds.clone(), &self.cols))
    }
}

impl<'vtab> CreateVTab<'vtab> for SQLTab {}

/// Sqlite 游标
#[repr(C)]
pub struct SQLTabCursor<'vtab> {
    base: ffi::sqlite3_vtab_cursor,
    /// 数据源实例
    ds: Arc<Box<dyn DataSource>>,
    /// 保存响应结果
    reader: Option<Response>,
    /// 保存列的定义
    columns: &'vtab Columns,
    /// 记录下一行结果
    next: Option<Row>,
    /// 记录行号
    rowid: usize,
    /// 是否为行结束
    eof: bool,
    phantom: PhantomData<&'vtab SQLTab>,
}

impl SQLTabCursor<'_> {
    fn new<'vtab>(ds: Arc<Box<dyn DataSource>>, cols: &'vtab Columns) -> SQLTabCursor<'vtab> {
        SQLTabCursor {
            base: ffi::sqlite3_vtab_cursor::default(),
            reader: None,
            next: None,
            rowid: 0,
            ds,
            columns: cols,
            eof: false,
            phantom: PhantomData,
        }
    }

    fn collect(&self, args: Args) -> Result<Response, Error> {
        let data_source: Arc<Box<dyn DataSource>> = self.ds.clone();
        let (mut request, statement) = new_req_none(args);
        // 执行请求
        let _ = async_std::task::spawn_blocking(move ||{
            if let Err(err) = data_source.collect(&mut request) {
                let _ = request.error(err);
            } else {
                let _ = request.ok();
            }
        });
        let resp = statement.wait()?;
        Ok(resp)
    }
}

unsafe impl VTabCursor for SQLTabCursor<'_> {
    fn filter(&mut self, _idx_num: c_int, _idx_str: Option<&str>, args: &Values<'_>) -> Result<()> {
        // 转换参数列表
        let values: Vec<rusqlite::types::FromSqlResult<Value>> =
            args.iter().map(|val| Value::column_result(val)).collect();
        let mut args = Args::new();
        for value in values {
            let val = value?;
            args.push(val);
        }
        debug!("args - {:?}", args);
        // 执行请求
        self.reader = Some(self.collect(args)?);
        self.rowid = 0;
        // 先获取一次结果行
        self.next()?;
        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        let reader: &Option<Response> = &self.reader;
        if let Some(reader) = reader {
            let option = reader.next_row();

            match option {
                Some(rs) => {
                    let row = rs?;
                    self.next = Some(row);
                    self.eof = false;
                    self.rowid += 1;
                }
                None => {
                    self.eof = true;
                }
            }
        } else {
            self.eof = true;
        }
        Ok(())
    }

    fn eof(&self) -> bool {
        self.eof
    }

    fn column(&self, ctx: &mut Context, col: c_int) -> Result<()> {
        // 获取列对应的结果值
        if let Some(row) = &self.next {
            let value: &Value = row.get_value(col as usize)?;
            return ctx.set_result(value);
        }

        Err(Error::ModuleError(format!(
            "column index out of bounds: {}",
            col
        )))
    }

    fn rowid(&self) -> Result<i64> {
        Ok(self.rowid as i64)
    }
}
