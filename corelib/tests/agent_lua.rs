mod common;

#[cfg(test)]
#[cfg(feature = "agent")]
#[cfg(feature = "lua")]
mod test {
    use crate::common::*;
    use std::time::Duration;

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
            0,
            Duration::from_secs(4),
        );
    }
}
