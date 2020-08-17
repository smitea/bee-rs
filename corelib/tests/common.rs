use bee_core::Connection;
#[cfg(test)]
pub fn init_log() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .try_init();
}
#[cfg(test)]
#[cfg(feature = "remote")]
#[cfg(feature = "sqlite")]
pub fn new_ssh_connection_for_sql() -> bee_core::Result<Box<dyn Connection>> {
    bee_core::new_connection(
        "sqlite:remote:password://oracle:admin@127.0.0.1:20002/bee?connect_timeout=5",
    )
}
#[cfg(test)]
#[cfg(feature = "agent")]
#[cfg(feature = "sqlite")]
pub fn new_agent_connection_for_sql() -> bee_core::Result<Box<dyn Connection>> {
    bee_core::new_connection("sqlite:agent:default")
}
#[cfg(test)]
#[cfg(feature = "remote")]
#[cfg(feature = "lua")]
pub fn new_ssh_connection_for_lua() -> bee_core::Result<Box<dyn Connection>> {
    bee_core::new_connection(
        "lua:remote:password://oracle:admin@127.0.0.1:20002/bee?connect_timeout=5",
    )
}
#[cfg(test)]
#[cfg(feature = "agent")]
#[cfg(feature = "lua")]
pub fn new_agent_connection_for_lua() -> bee_core::Result<Box<dyn Connection>> {
    bee_core::new_connection("lua:agent:default")
}
#[cfg(test)]
#[cfg(feature = "remote")]
#[cfg(feature = "sqlite")]
pub fn assert_remote_sql(
    sql: &str,
    columns: bee_core::Columns,
    row_size: usize,
    timeout: std::time::Duration,
) {
    let session: Box<dyn Connection> = new_ssh_connection_for_sql().unwrap();
    assert_columns(session, sql, columns, row_size, timeout);
}
#[cfg(test)]
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
#[cfg(test)]
#[cfg(feature = "remote")]
#[cfg(feature = "lua")]
pub fn assert_remote_lua(script: &str, row_size: usize, timeout: std::time::Duration) {
    let session: Box<dyn Connection> = new_ssh_connection_for_lua().unwrap();
    assert_row(session, script, row_size, timeout);
}
#[cfg(test)]
#[cfg(feature = "agent")]
#[cfg(feature = "lua")]
pub fn assert_agent_lua(script: &str, row_size: usize, timeout: std::time::Duration) {
    let session: Box<dyn Connection> = new_agent_connection_for_lua().unwrap();
    assert_row(session, script, row_size, timeout);
}
#[cfg(test)]
fn assert_row(
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
    println!("index: {}, row: {}",index,row_size);
    assert!(index >= row_size);
}
#[cfg(test)]
fn assert_columns(
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
        println!("row - {:?}",row);
        index += 1;
    }
    assert!(index >= row_size);
}