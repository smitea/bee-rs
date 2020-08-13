# Bee 测试工具

该项目旨在提供一组与大部分数据库提供的 CLI 一致的接口，便于观察测试脚本的输出结果。

Example: 

```shell
bee lua:agent:default 127.0.0.1:6142
```

切换并连接到数据源，可使用 SQL 语句:

```sql
use sqlite:agent:default;
```

查看最近一条执行的脚本信息可使用 SQL 语句:

```sql
show network_states;
```
