mod common;
#[cfg(test)]
#[cfg(feature = "agent")]
#[cfg(feature = "lua")]
mod test {
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
}