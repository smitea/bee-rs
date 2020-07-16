mod convert;
mod functions;
mod sql_tab;

use crate::Error;
use crate::{new_req, Args, Columns, DataType, Request, Row, Session, State, Statement, Value};
use async_std::task;
use convert::INVALIDCOLUMNCOUNT;
use functions::init_functions;
use parking_lot::*;
use rusqlite::vtab::read_only_module;
use rusqlite::{Connection, Result, NO_PARAMS};
use sql_tab::SQLTab;
use std::{sync::Arc, time::Duration};

pub struct SqliteSession {
    connection: Arc<Mutex<Connection>>,
}

impl Session for SqliteSession {
    fn register_source(&self, ds: Box<dyn crate::DataSource>) -> Result<(), Error> {
        let name = ds.name().to_string();
        let aux: Option<Arc<Box<dyn crate::DataSource>>> = Some(Arc::new(ds));
        let lock = self.connection.lock();
        lock.create_module(name.as_str(), read_only_module::<SQLTab>(), aux)?;
        Ok(())
    }

    fn query(&self, script: &str, timeout: Duration) -> Result<Statement, Error> {
        let (request, response) = new_req(Args::new(), timeout);
        let conn = self.connection.clone();
        let script = script.to_string();
        let _ = task::spawn(async move {
            let req = request;
            let rs = commit_statement(conn, script, &req);
            if let Err(err) = rs {
                let _ = req.error(err);
            }
        });

        Ok(response)
    }

    fn update(&self, script: &str, _timeout: Duration) -> Result<(), Error> {
        let conn = self.connection.clone();
        let lock = conn.lock();
        lock.execute_batch(script)?;
        Ok(())
    }
}

fn commit_statement(
    db: Arc<Mutex<Connection>>,
    script: String,
    request: &Request,
) -> Result<(), Error> {
    let lock = db.lock();
    let mut s = lock.prepare(script.as_str())?;
    let mut rows = s.query(NO_PARAMS)?;

    let sql_columns = rows.columns().ok_or(Error::invalid(
        INVALIDCOLUMNCOUNT,
        format!("can't find columns for sql[{}]", script),
    ))?;
    let mut columns = Columns::new();
    let count = sql_columns.len();

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

    let mut promise = request.head(columns)?;

    while let Ok(Some(rs)) = rows.next() {
        let mut row = Row::new();
        for i in 0..count {
            let val = rs.get::<usize, Value>(i)?;
            row.push(val);
        }
        promise.commit(State::from(row))?
    }
    Ok(())
}

pub fn new_session() -> Result<SqliteSession, Error> {
    let conn = Connection::open_in_memory()?;
    init_functions(&conn)?;
    Ok(SqliteSession {
        connection: Arc::new(Mutex::new(conn)),
    })
}

mod test {
    use super::new_session;
    use crate::{columns, row, Columns, DataSource, Promise, Session, State};
    use std::time::Duration;

    struct TestSource;

    impl DataSource for TestSource {
        fn name(&self) -> &str {
            "array"
        }

        fn columns(&self) -> Columns {
            columns![Integer: "v1"]
        }

        fn collect(&self, promise: &mut Promise) -> Result<(), crate::Error> {
            let args = promise.get_args();
            let count: i64 = args.get(0)?;
            for i in 0..count {
                promise.commit(State::from(row![i]))?;
            }
            Ok(())
        }
    }

    #[test]
    fn test_array() {
        let session = new_session().unwrap();
        let source1 = TestSource;
        session.register_source(Box::new(source1)).unwrap();

        let mut statements = vec![];

        let _ = session
            .update(
                "CREATE VIRTUAL TABLE vtab USING array(10)",
                Duration::from_secs(10),
            )
            .unwrap();

        for _ in 0..10000 {
            let statement = session
                .query("SELECT *FROM vtab", Duration::from_secs(10))
                .unwrap();
            statements.push(statement);
        }

        for statement in statements {
            let resp = statement.wait().unwrap();
            let columns = resp.columns();
            assert_eq!(1, columns.len());
            let mut index = 0;
            for rs in resp {
                let _ = rs.unwrap();
                index += 1;
            }
            assert_eq!(index, 10);
        }

        let _ = session
            .update("DROP TABLE vtab", Duration::from_secs(10))
            .unwrap();
    }

    #[test]
    fn test_ssh() {
        env_logger::init();
        // Filesystem     1K-blocks    Used Available Use% Mounted on
        // overlay         15312232 9295008   5219684  65% /
        // tmpfs              65536       8     65528   1% /dev
        // tmpfs            1018900       0   1018900   0% /sys/fs/cgroup
        // shm                65536       0     65536   0% /dev/shm
        // /dev/sda1       15312232 9295008   5219684  65% /etc/hosts
        // tmpfs            1018900       0   1018900   0% /proc/acpi
        // tmpfs              65536       8     65528   1% /proc/kcore
        // tmpfs              65536       8     65528   1% /proc/keys
        // tmpfs              65536       8     65528   1% /proc/timer_list
        // tmpfs              65536       8     65528   1% /proc/sched_debug
        // tmpfs            1018900       0   1018900   0% /sys/firmware
        let session: Box<dyn Session> = crate::new_session(
            "ssh://oracle:admin@127.0.0.1:49160/bee?connect_timeout=5&protocol=user_pwd",
        )
        .unwrap();

        let _ = session
            .update(
                "CREATE VIRTUAL TABLE vtab USING ssh('df -k',10)",
                Duration::from_secs(10),
            )
            .unwrap();

        let statement = session
            .query(
                r#"
                SELECT  
                    GET(output,0, 'TEXT', '') as filesystem,
                    GET(output,1, 'INT', 0) as total_bytes,
                    GET(output,2, 'INT', 0) as used_bytes,
                    GET(output,3, 'INT', 0) as avail_bytes
                FROM (SELECT SPLIT_SPACE(line) as output FROM vtab WHERE line NOT LIKE '%Filesystem%')
            "#,
                Duration::from_secs(4),
            )
            .unwrap();

        let resp = statement.wait().unwrap();
        let columns = resp.columns();
        assert_eq!(4, columns.len());
        println!("columns - {:?}", columns);
        let mut index = 0;
        for rs in resp {
            let _ = rs.unwrap();
            index += 1;
        }
        assert_eq!(index, 11);

        let _ = session
            .update("DROP TABLE vtab", Duration::from_secs(10))
            .unwrap();
    }
}
