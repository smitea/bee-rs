SELECT  get(output,0,'TEXT',0.0) as device,
        get(output,12,'REAL',0.0) as svctm,
        get(output,13,'REAL',0.0) as util
FROM (SELECT split_space(line) as output FROM shell('iostat -xk',10) WHERE line_num > 3)