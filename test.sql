-- noinspection SqlResolveForFile
CREATE VIEW HOST_SWAP AS
SELECT SUM(GET(output, 2, INT)) AS total_size,
       SUM(GET(output, 3, INT)) AS total_used
FROM (SELECT SPLIT_SPACE(line) AS output FROM SHELL('swapon -s', 'Filename', 1)) AS SRC;

CREATE VIEW HOST_MEMORY AS
SELECT buffers + cached + free AS free_bytes,
       total                   AS all_bytes,
       all_bytes - free_bytes  AS used_bytes
FROM (SELECT GET(output, 1, INT) AS total,
             GET(output, 3, INT) AS free,
             GET(output, 5, INT) AS buffers,
             GET(output, 6, INT) AS cached
      FROM (SELECT SPLIT_SPACE(line) AS output FROM SHELL('free -m', 'Mem:', 0)));

CREATE VIEW HOST_FILESYSTEM AS
SELECT GET(output, 0)             AS filesystem,
       GET(output, 2, INT) * 1024 AS used_bytes,
       GET(output, 3, INT) * 1024 AS free_bytes,
       used_bytes + free_bytes    AS all_bytes,
       GET(output, 5, INT)        AS mounted_on
FROM (SELECT SPLIT_SPACE(line) AS output FROM SHELL('df -Pk', '%Filesystem%', 1)) AS SRC;

CREATE VIEW HOST_BASIC AS
SELECT GET(output, 0, TEXT) AS name,
       GET(output, 1, TEXT) AS cpu,
       GET(output, 3, TEXT) AS cpu_mod,
       GET(output, 4, INT)  AS memory_mb,
       GET(output, 5, TEXT) AS up_time
FROM (SELECT SPLIT_CSV(line) AS output FROM PERL('${ORACLE_HOME}/bin/perl', '${BETHUNE_HOME}/BP_OS_Basic.pl')) AS SRC;

CREATE VIEW HOST_IOPS AS
SELECT LAST(output, 0, REAL) AS util_io_usage,
       LAST(output, 1, REAL) AS svctm_io_ms
FROM (SELECT SPLIT_SPACE(line) AS output FROM SHELL('iostat -xk', '%Device%', 1)) AS SRC;

CREATE VIEW OGG_SYNC_LAG AS
SELECT GET(output, 0)                                      AS program,
       GET(output, 1)                                      AS r_status,
       GET(output, 2)                                      AS group_name,
       GET(output, 3)                                      AS lag_at_chkt,
       GET(output, 4)                                      AS time_since_chkt,
       TIMESPAN(GET(output, 3)) + TIMESPAN(GET(output, 4)) AS delay_s
FROM (SELECT SPLIT_SPACE(line) AS output
      FROM SHELL('echo "info all"|$OGG_HOME/ggsci')
      WHERE line LIKE '%EXTRACT%'
         OR line LIKE '%REPLICAT%') as SRC;

