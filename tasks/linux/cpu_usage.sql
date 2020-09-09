SELECT  get(output,12,'REAL',0.0) as user,
        get(output,13,'REAL',0.0) as system,
        get(output,15,'REAL',0.0) as iowait,
        get(output,14,'REAL',0.0) as idle 
FROM (SELECT split_space(line) as output FROM shell('vmstat 1 2',10) WHERE line_num > 2)