mod common;

#[cfg(test)]
#[cfg(feature = "remote")]
#[cfg(feature = "sqlite")]
mod test{    
    use bee_core::*;
    use crate::common::*;
    use std::time::Duration;
    
    #[test]
    fn test_free_k() {
        init_log();
        assert_mock_sql(r#"
            SELECT  get(output,1,'INT',0) as used,
                    get(output,2,'INT',0) as free,
                    get(output,3,'INT',0) as shared,
                    get(output,4,'INT',0) as buffers,
                    get(output,5,'INT',0) as cached
            FROM (SELECT split_space(line) as output FROM shell('
                total       used       free     shared    buffers     cached
                Mem:       2037800    1104092     933708     189008      18664     684116
                -/+ buffers/cache:     401312    1636488
                Swap:      1048572          0    1048572
            ',10) 
            WHERE line LIKE '%Mem:%')
        "#, columns![Integer: "used", Integer: "free", Integer: "shared", Integer: "buffers", Integer: "cached"], 1,  Duration::from_secs(4));
    }
 
    #[test]
    fn test_vmstat_12() {
        init_log();
        assert_mock_sql(r#"
            SELECT  get(output,12,'REAL',0.0) as user,
                    get(output,13,'REAL',0.0) as system,
                    get(output,15,'REAL',0.0) as iowait,
                    get(output,14,'REAL',0.0) as idle 
            FROM (SELECT split_space(line) as output FROM shell('
                procs -----------memory---------- ---swap-- -----io---- -system-- ------cpu-----
                r  b   swpd   free   buff  cache   si   so    bi    bo   in   cs us sy id wa st
                1  0      0 855268  33484 741560    0    0    51    17  196  502  1  1 99  0  0
                0  0      0 855260  33484 741592    0    0     0    16  213  557  1  1 98  0  0
            ',10) WHERE line_num > 2)
        "#, columns![Number: "user", Number: "system", Number: "iowait", Number: "idle"], 1,  Duration::from_secs(4));
    }
    
    #[test]
    fn test_os() {
        init_log();
        assert_mock_sql(r#"
            SELECT line as os FROM shell('Linux',10)
        "#, columns![String: "os"], 1,  Duration::from_secs(4));
    }
}