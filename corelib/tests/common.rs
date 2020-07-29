use bee_core::Connection;

#[cfg(test)]
pub fn init_log() {
    let _ = env_logger::builder().is_test(true).filter_level(log::LevelFilter::Debug).try_init();
}

#[cfg(test)]
#[cfg(feature = "remote")]
pub fn new_ssh_connection() -> bee_core::Result<bee_core::SqliteSession> {
    bee_core::new_connection(
        "remote://oracle:admin@127.0.0.1:49160/bee?connect_timeout=5&protocol=user_pwd",
    )
}

#[cfg(test)]
#[cfg(feature = "agent")]
pub fn new_agent_connection() -> bee_core::Result<bee_core::SqliteSession> {
    bee_core::new_connection(
        "agent://127.0.0.1:6142",
    )
}

#[cfg(test)]
#[cfg(feature = "remote")]
pub fn assert_remote_sql(sql: &str,columns: bee_core::Columns, row_size: usize, timeout: std::time::Duration){
    let session: bee_core::SqliteSession = new_ssh_connection().unwrap();
    assert_sql(session, sql, columns, row_size, timeout);
} 

#[cfg(test)]
#[cfg(feature = "agent")]
pub fn assert_agent_sql(sql: &str, columns: bee_core::Columns, row_size: usize, timeout: std::time::Duration){
    let session: bee_core::SqliteSession = new_agent_connection().unwrap();
    assert_sql(session, sql, columns, row_size, timeout);
}

#[cfg(test)]
fn assert_sql<T: Connection>(session: T, sql: &str, columns: bee_core::Columns, row_size: usize, timeout: std::time::Duration){
    let statement = session.new_statement(sql,timeout).unwrap();
    let resp = statement.wait().unwrap();
    let new_columns = resp.columns();
    assert_eq!(&columns,new_columns);
    let mut index = 0;
    for rs in resp {
        let _ = rs.unwrap();
        index += 1;
    }
    assert_eq!(index, row_size);
}