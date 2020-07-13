use crate::Value;
use crate::{new_req, Args, DataSource, Response, Row};
use async_std::task;
use rusqlite::ffi;
use rusqlite::vtab::{Context, CreateVTab, IndexInfo, VTab, VTabConnection, VTabCursor, Values};
use rusqlite::{types::Null, Error, Result};
use std::marker::PhantomData;
use std::{os::raw::c_int, sync::Arc};

#[repr(C)]
pub struct SQLTab {
    /// Base class. Must be first
    base: ffi::sqlite3_vtab,
    response: Response,
    offset_first_row: usize,
}

unsafe impl<'vtab> VTab<'vtab> for SQLTab {
    type Aux = Arc<Box<dyn DataSource>>;
    type Cursor = SQLTabCursor<'vtab>;

    fn best_index(&self, info: &mut IndexInfo) -> Result<()> {
        info.set_estimated_cost(1_000_000.);
        Ok(())
    }

    fn open(&self) -> Result<SQLTabCursor<'_>> {
        Ok(SQLTabCursor::new(&self.response))
    }

    fn connect(
        _: &mut VTabConnection,
        aux: Option<&Self::Aux>,
        args: &[&[u8]],
    ) -> Result<(String, Self)> {
        let mut a = Args::new();
        let args = &args[3..];
        for arg in args {
            let arg = std::str::from_utf8(arg)?.trim();
            a.push(arg.parse::<Value>()?);
        }
        let ds = match aux {
            Some(ds) => ds,
            None => return Err(Error::ModuleError("No datasource".to_owned())),
        };

        let data_source: Arc<Box<dyn DataSource>> = ds.clone();
        let (request, statement) = new_req(a, None);

        let _ = task::spawn(async move {
            let rs = data_source.collect(&request);
            if let Err(err) = rs {
                if let Err(err) = request.error(err) {
                    println!("err - {}", err);
                }
            }
        });

        let resp = statement.wait()?;
        let columns = resp.columns();

        let name = ds.name();

        let mut sql = format!("CREATE TABLE {}(", name);

        for (index, col) in columns.iter().enumerate() {
            sql.push('"');
            sql.push_str(&col.0);
            sql.push_str("\" ");

            match col.1 {
                crate::DataType::String => sql.push_str("TEXT"),
                crate::DataType::Integer => sql.push_str("INTEGER"),
                crate::DataType::Number => sql.push_str("REAL"),
                crate::DataType::Boolean => sql.push_str("INTEGER"),
                crate::DataType::Bytes => sql.push_str("BLOB"),
                crate::DataType::Nil => sql.push_str("NULL"),
            };
            if index == columns.len() - 1 {
                sql.push_str(");");
            } else {
                sql.push_str(", ");
            }
        }

        let vtab = SQLTab {
            base: ffi::sqlite3_vtab::default(),
            response: resp,
            offset_first_row: 0,
        };

        Ok((sql, vtab))
    }
}

impl<'vtab> CreateVTab<'vtab> for SQLTab {}

#[repr(C)]
pub struct SQLTabCursor<'vtab> {
    base: ffi::sqlite3_vtab_cursor,
    reader: &'vtab Response,
    next: Option<Row>,
    rowid: usize,
    eof: bool,
    phantom: PhantomData<&'vtab SQLTab>,
}

impl SQLTabCursor<'_> {
    fn new<'vtab>(reader: &'vtab Response) -> SQLTabCursor<'vtab> {
        SQLTabCursor {
            base: ffi::sqlite3_vtab_cursor::default(),
            reader,
            next: None,
            rowid: 0,
            eof: false,
            phantom: PhantomData,
        }
    }
}

unsafe impl VTabCursor for SQLTabCursor<'_> {
    fn filter(
        &mut self,
        _idx_num: c_int,
        _idx_str: Option<&str>,
        _args: &Values<'_>,
    ) -> Result<()> {
        self.rowid = 0;
        self.next()
    }

    fn next(&mut self) -> Result<()> {
        let reader: &Response = &self.reader;
        let option = reader.next_row();

        match option {
            Some(rs) => {
                let row = rs?;
                self.next = Some(row);
                self.eof = false;
            }
            None => {
                self.eof = false;
            }
        }
        self.rowid += 1;
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
