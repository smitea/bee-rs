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
            SELECT * FROM filesystem() WHERE name NOT LIKE '%tmp%'
        "#, columns![String: "name", String: "mount_on", Integer: "total_bytes", Integer: "used_bytes", Integer: "free_bytes"], 2,  Duration::from_secs(4));
    }

    #[test]
    fn host_basic(){
        init_log();
        assert_agent_sql(r#"
            SELECT * FROM host_basic()
        "#, columns![String: "host_name", Integer: "cpu_core", String: "cpu_model", Integer: "uptime", Integer: "memory"], 1,  Duration::from_secs(4));
    }

    #[test]
    fn cpu_usage(){
        init_log();
        assert_agent_sql(r#"
            SELECT * FROM cpu_usage()
        "#, columns![Number: "idle", Number: "user", Number: "system", Number: "iowait"], 1,  Duration::from_secs(4));
    }

    #[test]
    fn os_info(){
        init_log();
        assert_agent_sql(r#"
            SELECT * FROM os_info()
        "#, columns![String: "os_type", String: "version", String: "host_name"], 1,  Duration::from_secs(4));
    }

    #[test]
    fn memory_usage(){
        init_log();
        assert_agent_sql(r#"
            SELECT * FROM memory_usage()
        "#, columns![Integer: "used_bytes", Integer: "total_bytes", Integer: "free_bytes"], 1,  Duration::from_secs(4));
    }

    #[test]
    fn swap_usage(){
        init_log();
        assert_agent_sql(r#"
            SELECT * FROM swap_usage()
        "#, columns![Integer: "used_bytes", Integer: "total_bytes", Integer: "free_bytes"], 1,  Duration::from_secs(4));
    }
}