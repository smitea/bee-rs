use bee_core::Connection;

pub fn init_log() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .try_init();
}

#[allow(dead_code)]
#[cfg(feature = "agent")]
#[cfg(feature = "sqlite")]
pub fn new_agent_connection_for_sql() -> bee_core::Result<Box<dyn Connection>> {
    bee_core::new_connection("sqlite:agent:default")
}

#[allow(dead_code)]
#[cfg(feature = "agent")]
#[cfg(feature = "lua")]
pub fn new_agent_connection_for_lua() -> bee_core::Result<Box<dyn Connection>> {
    bee_core::new_connection("lua:agent:default")
}

#[allow(dead_code)]
#[cfg(feature = "agent")]
#[cfg(feature = "sqlite")]
pub fn assert_agent_sql(
    sql: &str,
    columns: bee_core::Columns,
    row_size: usize,
    timeout: std::time::Duration,
) {
    let session: Box<dyn Connection> = new_agent_connection_for_sql().unwrap();
    assert_columns(session, sql, columns, row_size, timeout);
}

#[allow(dead_code)]
#[cfg(feature = "agent")]
#[cfg(feature = "lua")]
pub fn assert_agent_lua(script: &str, row_size: usize, timeout: std::time::Duration) {
    let session: Box<dyn Connection> = new_agent_connection_for_lua().unwrap();
    assert_row(session, script, row_size, timeout);
}

#[allow(dead_code)]
pub fn assert_row(
    session: Box<dyn Connection>,
    sql: &str,
    row_size: usize,
    timeout: std::time::Duration,
) {
    let statement = session.new_statement(sql, timeout).unwrap();
    let resp = statement.wait().unwrap();
    let mut index = 0;
    for rs in resp {
        let _ = rs.unwrap();
        index += 1;
    }
    println!("index: {}, row: {}", index, row_size);
    assert!(index >= row_size);
}

#[allow(dead_code)]
pub fn assert_columns(
    session: Box<dyn Connection>,
    sql: &str,
    columns: bee_core::Columns,
    row_size: usize,
    timeout: std::time::Duration,
) {
    let statement = session.new_statement(sql, timeout).unwrap();
    let resp = statement.wait().unwrap();
    let new_columns = resp.columns();
    assert_eq!(&columns, new_columns);
    let mut index = 0;
    for rs in resp {
        let row = rs.unwrap();
        println!("row - {:?}", row);
        index += 1;
    }
    assert!(index >= row_size);
}
