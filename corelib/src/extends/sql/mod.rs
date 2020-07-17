mod convert;
mod functions;
mod sql_tab;

use crate::Error;
use crate::{new_req, Args, Columns, DataType, Request, Session, State, Statement, Value};
use async_std::task;
use convert::INVALIDCOLUMNCOUNT;
use functions::init_functions;
use parking_lot::*;
use rusqlite::vtab::eponymous_only_module;
use rusqlite::{Column, Connection, Result, Row, NO_PARAMS};
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
        lock.create_module(name.as_str(), eponymous_only_module::<SQLTab>(), aux)?;
        println!("register ds - {}", name);
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

    // 尝试获取一行数据，才能决定列的类型
    let mut promise = if let Ok(Some(row)) = rows.next() {
        let new_row = get_row(row)?;
        let mut cols = Columns::new();
        for i in 0..row.column_count() {
            let name = row.column_name(i)?;
            let value = row.get::<usize, Value>(i)?;

            cols.push(name, DataType::from(value));
        }

        let mut promise = request.head(cols)?;
        promise.commit(State::from(new_row))?;
        promise
    } else {
        let sql_columns = rows.columns().ok_or(Error::invalid(
            INVALIDCOLUMNCOUNT,
            format!("can't find columns for sql[{}]", script),
        ))?;
        request.head(get_columns(sql_columns))?
    };

    while let Ok(Some(rs)) = rows.next() {
        promise.commit(State::from(get_row(rs)?))?;
    }
    Ok(())
}

fn get_row(rs: &Row) -> Result<crate::Row, Error> {
    let count = rs.column_count();
    let mut row = crate::Row::new();
    for i in 0..count {
        let val = rs.get::<usize, Value>(i)?;
        row.push(val);
    }
    Ok(row)
}

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

pub fn new_session() -> Result<SqliteSession, Error> {
    let conn = Connection::open_in_memory()?;
    init_functions(&conn)?;
    Ok(SqliteSession {
        connection: Arc::new(Mutex::new(conn)),
    })
}

mod test {
    use crate::{columns, row, Columns, DataSource, Promise, Session, State};

    struct TestSource;

    impl DataSource for TestSource {
        fn name(&self) -> &str {
            "ARRAY"
        }

        fn args(&self) -> Columns {
            columns![Integer: "count"]
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
        use std::time::Duration;

        let session = super::new_session().unwrap();
        let source1 = TestSource;
        session.register_source(Box::new(source1)).unwrap();

        let mut statements = vec![];

        for _ in 0..10000 {
            let statement = session
                .query("SELECT *FROM ARRAY(10)", Duration::from_secs(10))
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
    }

    #[test]
    fn test_df_k() {
        use std::time::Duration;

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

        let statement = session
            .query(
                r#"
                SELECT  get(output,0,'TEXT','') as filesystem,
                        get(output,1,'INT',0) as total,
                        get(output,2,'INT',0) as used,
                        get(output,3,'INT',0) as avail
                FROM (SELECT split_space(line) as output FROM ssh('df -k',10) 
                WHERE line NOT LIKE '%Filesystem%' AND line NOT LIKE '%tmp%')
            "#,
                Duration::from_secs(4),
            )
            .unwrap();

        let resp = statement.wait().unwrap();
        let columns = resp.columns();
        assert_eq!(4, columns.len());
        assert_eq!(
            &columns![String: "filesystem", Integer: "total", Integer: "used", Integer: "avail"],
            columns
        );
        println!("columns - {:?}",columns);
        let mut index = 0;
        for rs in resp {
            let row = rs.unwrap();
            println!("row - {:?}",row);
            index += 1;
        }
        assert_eq!(index, 3);
    }

    #[test]
    fn test_free_k() {
        use std::time::Duration;

        // total       used       free     shared    buffers     cached
        // Mem:       2037800    1104092     933708     189008      18664     684116
        // -/+ buffers/cache:     401312    1636488
        // Swap:      1048572          0    1048572
        let session: Box<dyn Session> = crate::new_session(
            "ssh://oracle:admin@127.0.0.1:49160/bee?connect_timeout=5&protocol=user_pwd",
        )
        .unwrap();

        let statement = session
            .query(
                r#"
                SELECT  get(output,1,'INT',0) as used,
                        get(output,2,'INT',0) as free,
                        get(output,3,'INT',0) as shared,
                        get(output,4,'INT',0) as buffers,
                        get(output,5,'INT',0) as cached
                FROM (SELECT split_space(line) as output FROM ssh('free -k',10) 
                WHERE line LIKE '%Mem:%')
            "#,
                Duration::from_secs(4),
            )
            .unwrap();

        let resp = statement.wait().unwrap();
        let columns = resp.columns();
        assert_eq!(5, columns.len());
        assert_eq!(
            &columns![Integer: "used", Integer: "free", Integer: "shared", Integer: "buffers", Integer: "cached"],
            columns
        );
        println!("columns - {:?}",columns);
        let mut index = 0;
        for rs in resp {
            let row = rs.unwrap();
            println!("row - {:?}",row);
            index += 1;
        }
        assert_eq!(index, 1);
    }

    #[test]
    fn test_iostat_xk() {
        use std::time::Duration;

        // Linux 4.19.76-linuxkit (cb9607b8c76e) 	07/17/20 	_x86_64_	(1 CPU)
        // avg-cpu:  %user   %nice %system %iowait  %steal   %idle
        //            0.68    0.00    0.72    0.05    0.00   98.55
        // Device:         rrqm/s   wrqm/s     r/s     w/s    rkB/s    wkB/s avgrq-sz avgqu-sz   await r_await w_await  svctm  %util
        // scd0              0.00     0.00    0.23    0.00    16.64     0.00   147.46     0.00    0.60    0.60    0.00   0.46   0.01
        // sda               0.00     0.63    1.00    1.74    24.87    16.51    30.23     0.00    0.48    0.39    0.53   0.33   0.09
        // scd1              0.00     0.00    0.00    0.00     0.01     0.00    36.57     0.00    0.14    0.14    0.00   0.14   0.00
        // scd2              0.00     0.00    0.11    0.00     9.03     0.00   160.65     0.00    0.70    0.70    0.00   0.66   0.01
        let session: Box<dyn Session> = crate::new_session(
            "ssh://oracle:admin@127.0.0.1:49160/bee?connect_timeout=5&protocol=user_pwd",
        )
        .unwrap();

        let statement = session
            .query(
                r#"
                SELECT  get(output,0,'TEXT',0.0) as device,
                        get(output,12,'REAL',0.0) as svctm,
                        get(output,13,'REAL',0.0) as util
                FROM (SELECT split_space(line) as output FROM ssh('iostat -xk',10) WHERE line_num > 3)
            "#,
                Duration::from_secs(4),
            )
            .unwrap();

        let resp = statement.wait().unwrap();
        let columns = resp.columns();
        assert_eq!(3, columns.len());
        assert_eq!(
            &columns![String: "device", Number: "svctm", Number: "util"],
            columns
        );
        println!("columns - {:?}",columns);
        let mut index = 0;
        for rs in resp {
            let row = rs.unwrap();
            println!("row - {:?}",row);
            index += 1;
        }
        assert_eq!(index, 4);
    }

    #[test]
    fn test_vmstat_12() {
        use std::time::Duration;

        // procs -----------memory---------- ---swap-- -----io---- -system-- ------cpu-----
        // r  b   swpd   free   buff  cache   si   so    bi    bo   in   cs us sy id wa st
        // 1  0      0 855268  33484 741560    0    0    51    17  196  502  1  1 99  0  0
        // 0  0      0 855260  33484 741592    0    0     0    16  213  557  1  1 98  0  0
        let session: Box<dyn Session> = crate::new_session(
            "ssh://oracle:admin@127.0.0.1:49160/bee?connect_timeout=5&protocol=user_pwd",
        )
        .unwrap();

        let statement = session
            .query(
                r#"
                SELECT  get(output,12,'REAL',0.0) as user,
                        get(output,13,'REAL',0.0) as system,
                        get(output,15,'REAL',0.0) as iowait,
                        get(output,14,'REAL',0.0) as idle 
                FROM (SELECT split_space(line) as output FROM ssh('vmstat 1 2',10) WHERE line_num > 2)
            "#,
                Duration::from_secs(4),
            )
            .unwrap();

        let resp = statement.wait().unwrap();
        let columns = resp.columns();
        assert_eq!(4, columns.len());
        assert_eq!(
            &columns![Number: "user", Number: "system", Number: "iowait", Number: "idle"],
            columns
        );
        println!("columns - {:?}",columns);
        let mut index = 0;
        for rs in resp {
            let row = rs.unwrap();
            println!("row - {:?}",row);
            index += 1;
        }
        assert_eq!(index, 1);
    }

    #[test]
    fn test_swapon_s(){
        use std::time::Duration;
        // Filename				Type		Size	Used	Priority
        // /swap                file		1048572	0	    -2
        let session: Box<dyn Session> = crate::new_session(
            "ssh://oracle:admin@127.0.0.1:49160/bee?connect_timeout=5&protocol=user_pwd",
        )
        .unwrap();

        let statement = session
            .query(
                r#"
                SELECT file_name,total,used, total - used as avali FROM (
                    SELECT  get(output,0,'TEXT',0) as file_name,
                        get(output,2,'INT',0) as total,
                        get(output,3,'INT',0) as used
                    FROM (SELECT split_space(line) as output FROM ssh('swapon -s',10) WHERE line_num > 0)
                )
            "#,
                Duration::from_secs(4),
            )
            .unwrap();

        let resp = statement.wait().unwrap();
        let columns = resp.columns();
        assert_eq!(4, columns.len());
        assert_eq!(
            &columns![String: "file_name", Integer: "total", Integer: "used", Integer: "avali"],
            columns
        );
        println!("columns - {:?}",columns);
        let mut index = 0;
        for rs in resp {
            let row = rs.unwrap();
            println!("row - {:?}",row);
            index += 1;
        }
        assert_eq!(index, 1);
    }

    #[test]
    fn test_os(){
        use std::time::Duration;
        let session: Box<dyn Session> = crate::new_session(
            "ssh://oracle:admin@127.0.0.1:49160/bee?connect_timeout=5&protocol=user_pwd",
        )
        .unwrap();

        let statement = session
            .query(
                r#"
                SELECT line as os FROM ssh('perl -e "print($^O)"',10)
            "#,
                Duration::from_secs(4),
            )
            .unwrap();

        let resp = statement.wait().unwrap();
        let columns = resp.columns();
        assert_eq!(1, columns.len());
        assert_eq!(
            &columns![String: "os"],
            columns
        );
        println!("columns - {:?}",columns);
        let mut index = 0;
        for rs in resp {
            let row = rs.unwrap();
            println!("row - {:?}",row);
            index += 1;
        }
        assert_eq!(index, 1);
    }
}
