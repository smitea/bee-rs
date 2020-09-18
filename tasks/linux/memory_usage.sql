SELECT  get(output,1,'INT',0) as used,
        get(output,2,'INT',0) as free,
        get(output,3,'INT',0) as shared,
        get(output,4,'INT',0) as buffers,
        get(output,5,'INT',0) as cached
FROM (SELECT split_space(line) as output FROM shell('free -k',10) 
WHERE line LIKE '%Mem:%')