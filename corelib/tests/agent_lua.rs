mod common;
#[cfg(test)]
#[cfg(feature = "agent")]
#[cfg(feature = "lua")]
mod lua_test {
    use crate::common::*;
    use std::time::Duration;
    #[test]
    fn filesystem() {
        init_log();
        assert_agent_lua(
            r#"
            local resp=filesystem();
            while(resp:has_next())
            do
                _request:commit(_next);
            end
        "#,
            3,
            Duration::from_secs(4),
        );
    }
    #[test]
    fn host_basic() {
        init_log();
        assert_agent_lua(
            r#"
            local resp=host_basic();
            while(resp:has_next())
            do
                _request:commit(_next);
            end
        "#,
            1,
            Duration::from_secs(4),
        );
    }
    #[test]
    fn cpu_usage() {
        init_log();
        assert_agent_lua(
            r#"
            local resp=cpu_usage();
            while(resp:has_next())
            do
                _request:commit(_next);
            end
        "#,
            1,
            Duration::from_secs(4),
        );
    }
    #[test]
    fn os_info() {
        init_log();
        assert_agent_lua(
            r#"
            local resp=os_info();
            while(resp:has_next())
            do
                _request:commit(_next);
            end
        "#,
            1,
            Duration::from_secs(4),
        );
    }
    #[test]
    fn memory_usage() {
        init_log();
        assert_agent_lua(
            r#"
            local resp=memory_usage();
            while(resp:has_next())
            do
                _request:commit(_next);
            end
        "#,
            1,
            Duration::from_secs(4),
        );
    }
    #[test]
    fn swap_usage() {
        init_log();
        assert_agent_lua(
            r#"
            local resp=swap_usage();
            while(resp:has_next())
            do
                _request:commit(_next);
            end
        "#,
            1,
            Duration::from_secs(4),
        );
    }
    #[test]
    fn shell() {
        init_log();
        assert_agent_lua(
            r#"
            local resp=shell("echo Hello", 10)
            while(resp:has_next())
            do
                if(_next["line_num"] == 0) then
                    _request:commit(_next);
                end
            end
        "#,
            1,
            Duration::from_secs(4),
        );
    }
}
