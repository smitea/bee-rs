use crate::Value;
use crate::{new_req_none, Args, Columns, DataSource, Request, Response, Row};
use rusqlite::ffi;
use rusqlite::vtab::{
    Context, CreateVTab, IndexConstraintOp, IndexInfo, VTab, VTabConnection, VTabCursor, Values,
};
use rusqlite::{types::FromSql, types::FromSqlError, Error, Result};
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

fn collect(
    data_source: Arc<Box<dyn DataSource>>,
    request: &Request,
    cols: Columns,
) -> Result<(), crate::Error> {
    let mut promise = request.head(cols)?;

    data_source.collect(&mut promise)?;
    Ok(())
}

unsafe impl<'vtab> VTab<'vtab> for SQLTab {
    type Aux = Arc<Box<dyn DataSource>>;
    type Cursor = SQLTabCursor<'vtab>;

    fn open(&self) -> Result<SQLTabCursor<'_>> {
        Ok(SQLTabCursor::new(self.ds.clone(), &self.cols))
    }

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
        let mut sql = format!("CREATE TABLE {}(", name);

        for (index, col) in columns.iter().enumerate() {
            sql.push_str(&col.0);
            sql.push_str(" ");

            match col.1 {
                crate::DataType::String => sql.push_str("TEXT"),
                crate::DataType::Integer => sql.push_str("INTEGER"),
                crate::DataType::Number => sql.push_str("REAL"),
                crate::DataType::Boolean => sql.push_str("INTEGER"),
                crate::DataType::Bytes => sql.push_str("BLOB"),
                crate::DataType::Nil => sql.push_str("NULL"),
            };
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

            match col.1 {
                crate::DataType::String => sql.push_str("TEXT"),
                crate::DataType::Integer => sql.push_str("INTEGER"),
                crate::DataType::Number => sql.push_str("REAL"),
                crate::DataType::Boolean => sql.push_str("INTEGER"),
                crate::DataType::Bytes => sql.push_str("BLOB"),
                crate::DataType::Nil => sql.push_str("NULL"),
            };
            if index < args.len() - 1 {
                sql.push_str(", ");
            }
        }

        sql.push_str(");");

        println!("Custom SQL: {}", sql);

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
        let cols_index = self.cols.len();
        for (i, constraint) in info.constraints().enumerate() {
            if !constraint.is_usable() {
                continue;
            }
            if constraint.operator() != IndexConstraintOp::SQLITE_INDEX_CONSTRAINT_EQ {
                continue;
            }
            let param_index = constraint.column() as usize - cols_index;
            params.push(param_index);
            idx_num |= 1 << i;
        }

        let mut num_of_arg = 0;
        for index in params {
            num_of_arg += 1;
            let mut constraint_usage = info.constraint_usage(index);
            constraint_usage.set_argv_index(num_of_arg);
            constraint_usage.set_omit(true);
        }

        info.set_estimated_cost(2_147_483_647f64);
        info.set_estimated_rows(2_147_483_647);
        info.set_idx_num(idx_num as i32);
        Ok(())
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
        let cols = self.ds.columns();
        let (request, statement) = new_req_none(args);
        collect(data_source, &request, cols)?;
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
            println!("none resp");
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
