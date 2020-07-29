mod common;

#[cfg(feature = "agent")]
#[cfg(test)]
mod test{
    use bee_core::*;
    use crate::common::*;
    use std::time::Duration;

    #[test]
    fn filesystem(){
        init_log();
        assert_agent_sql(r#"
            SELECT *FROM filesystem()
        "#, columns![String: "filesystem", Integer: "total", Integer: "used", Integer: "avail"], 3,  Duration::from_secs(4));
    }
}