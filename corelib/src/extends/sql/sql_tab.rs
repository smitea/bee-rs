use crate::Value;
use crate::{new_req_none, Args, Columns, DataSource, Request, Response, Row};
use rusqlite::ffi;
use rusqlite::vtab::{
    dequote, Context, CreateVTab, IndexInfo, VTab, VTabConnection, VTabCursor, Values,
};
use rusqlite::{Error, Result};
use std::marker::PhantomData;
use std::{os::raw::c_int, sync::Arc};

#[repr(C)]
pub struct SQLTab {
    /// Base class. Must be first
    base: ffi::sqlite3_vtab,
    ds: Arc<Box<dyn DataSource>>,
    args: Args,
    offset_first_row: usize,
}

impl SQLTab {
    fn collect(&self) -> Result<Response, Error> {
        let data_source: Arc<Box<dyn DataSource>> = self.ds.clone();
        let cols = self.ds.columns();
        let (request, statement) = new_req_none(self.args.clone());
        // 异步执行，可
        std::thread::spawn(move || {
            if let Err(err) = collect(data_source, &request, cols) {
                println!("has a err - {}", err);
                let _ = request.error(err);
            }
        });

        let resp = statement.wait()?;
        Ok(resp)
    }
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

    fn best_index(&self, info: &mut IndexInfo) -> Result<()> {
        info.set_estimated_cost(1_000_000.);
        Ok(())
    }

    fn open(&self) -> Result<SQLTabCursor<'_>> {
        let response = self.collect()?;
        Ok(SQLTabCursor::new(response))
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
            let arg = dequote(arg);
            a.push(arg.parse::<Value>()?);
        }
        let ds = match aux {
            Some(ds) => ds,
            None => return Err(Error::ModuleError("No datasource".to_owned())),
        };

        let name = ds.name();
        let columns = ds.columns();

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
            ds: ds.clone(),
            offset_first_row: 0,
            args: a,
        };

        Ok((sql, vtab))
    }
}

impl<'vtab> CreateVTab<'vtab> for SQLTab {}

#[repr(C)]
pub struct SQLTabCursor<'vtab> {
    base: ffi::sqlite3_vtab_cursor,
    reader: Response,
    next: Option<Row>,
    rowid: usize,
    eof: bool,
    phantom: PhantomData<&'vtab SQLTab>,
}

impl SQLTabCursor<'_> {
    fn new<'vtab>(reader: Response) -> SQLTabCursor<'vtab> {
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
        self.next()?;
        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        let reader: &Response = &self.reader;
        let option = reader.next_row();

        match option {
            Some(rs) => {
                println!("rs - {:?}",rs);
                let row = rs?;
                self.next = Some(row);
                self.eof = false;
                self.rowid += 1;
            }
            None => {
                self.eof = true;
            }
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
