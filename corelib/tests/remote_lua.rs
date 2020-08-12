mod common;

#[cfg(test)]
#[cfg(feature = "remote")]
#[cfg(feature = "lua")]
mod test{    
    use crate::common::*;
    use std::time::Duration;
    
    #[test]
    fn test_df_k() {
        init_log();
    
        // Filesystem     1K-blocks    Used Available Use% Mounted on
        // overlay         15312232 9295008   5219684  65% /
        // tmpfs              65536       8     65528   1% /dev
        // tmpfs            1018900       0   1018900   0% /sys/fs/cgroup
        // shm                65536       0     65536   0% /dev/shm
        // /dev/sda1       15312232 9295008   5219684  65% /etc/hosts
        // tmpfs            1018900       0   1018900   0% /proc/acpi
        // tmpfs              65536       8     65528   1% /proc/kcore
        // tmpfs              65536       8     65528   1% /proc/keys
        // tmpfs              65536       8     65528   1% /proc/timer_list
        // tmpfs              65536       8     65528   1% /proc/sched_debug
        // tmpfs            1018900       0   1018900   0% /sys/firmware
        assert_remote_lua(r#"
            local resp = remote_shell("df -k",10)
            while(resp:has_next())
            do
                local line = _next["line"]
                local line_num = _next["line_num"]
                if(line_num > 0) then
                    local cols = split_space(line)
                    _request:commit({
                        filesystem  = get(cols,0,"TEXT",""),
                        total       = get(cols,1,"INT",0),
                        used        = get(cols,2,"INT",0),
                        avail       = get(cols,3,"INT",0)
                    })
                end
            end
        "#, 3,  Duration::from_secs(4));
    }
}