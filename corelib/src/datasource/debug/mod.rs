use crate::{Configure, Instance, Result};

mod shell;

/// 注册数据源
pub fn register_ds<T: Configure>(_: &Instance, connection: &T) -> Result<()> {
    use crate::register_ds;
    connection.register_source(register_ds!(shell))?;
    Ok(())
}

#[test]
fn test() {
    let _ = crate::new_connection("sqlite:debug:default").unwrap();
}
