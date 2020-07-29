use crate::Value;
use crate::{new_req_none, Args, Columns, DataSource, DataType, Response, Row};
use rusqlite::ffi;
use rusqlite::vtab::{
    Context, CreateVTab, IndexConstraintOp, IndexInfo, VTab, VTabConnection, VTabCursor, Values,
};
use rusqlite::{types::FromSql, Error, Result};
use std::marker::PhantomData;
use std::{os::raw::c_int, sync::Arc};

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

fn to_dml_sql(name: &str, args: &Columns, columns: &Columns) -> String {
    let mut sql = format!("CREATE TABLE {}(", name);

    for (index, col) in columns.iter().enumerate() {
        sql.push_str(&col.0);
        sql.push_str(" ");

        sql.push_str(to_sqlite_type(col.1));
        if index < columns.len() - 1 {
            sql.push_str(", ");
        }
    }

    if columns.len() > 0 {
        sql.push(',');
    }

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
        let ds = match aux {
            Some(ds) => ds,
            None => return Err(Error::ModuleError("No datasource".to_owned())),
        };

        let columns = ds.columns();
        let args = ds.args();
        let name = ds.name();

        let sql = to_dml_sql(name, &args, &columns);

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
        let mut idx_num = 0;
        let mut params = vec![];
        let cols_len = self.cols.len() as i32;
        for (i, constraint) in info.constraints().enumerate() {
            if !constraint.is_usable() {
                continue;
            }

            if constraint.operator() == IndexConstraintOp::SQLITE_INDEX_CONSTRAINT_EQ {
                if constraint.column() >= cols_len {
                    params.push(i);
                    idx_num |= 1 << constraint.column();
                }
            }
        }

        let mut num_of_arg = 0;
        for index in params {
            num_of_arg += 1;
            let mut constraint_usage = info.constraint_usage(index);
            constraint_usage.set_argv_index(num_of_arg);
            constraint_usage.set_omit(true);
        }
        info.set_idx_num(idx_num as i32);
        Ok(())
    }

    fn open(&self) -> Result<SQLTabCursor<'_>> {
        Ok(SQLTabCursor::new(self.ds.clone(), &self.cols))
    }
}

impl<'vtab> CreateVTab<'vtab> for SQLTab {}

#[repr(C)]
pub struct SQLTabCursor<'vtab> {
    base: ffi::sqlite3_vtab_cursor,
    ds: Arc<Box<dyn DataSource>>,
    reader: Option<Response>,
    columns: &'vtab Columns,
    next: Option<Row>,
    rowid: usize,
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
        data_source.collect(&mut request)?;
        let resp = statement.wait()?;
        Ok(resp)
    }
}

unsafe impl VTabCursor for SQLTabCursor<'_> {
    fn filter(&mut self, _idx_num: c_int, _idx_str: Option<&str>, args: &Values<'_>) -> Result<()> {
        let values: Vec<rusqlite::types::FromSqlResult<Value>> =
            args.iter().map(|val| Value::column_result(val)).collect();

        let mut args = Args::new();
        for value in values {
            let val = value?;
            args.push(val);
        }

        self.reader = Some(self.collect(args)?);
        self.rowid = 0;
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
            self.eof = false;
        }
        Ok(())
    }

    fn eof(&self) -> bool {
        self.eof
    }

    fn column(&self, ctx: &mut Context, col: c_int) -> Result<()> {
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
