mod common;

#[cfg(test)]
#[cfg(feature = "agent")]
#[cfg(feature = "lua")]
mod test {
    use crate::common::*;
    use std::time::Duration;

    #[test]
    fn test_commit() {
        init_log();
        assert_agent_lua(
            r#"
            _request:commit({
                name = "He"
            });
            "#,
            0,
            Duration::from_secs(4),
        );
    }
}
