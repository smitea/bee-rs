# Bee Core

Bee 采用一种基于脚本的数据解析方式，可使用 Lua/SQL 来对主机监控数据爬取和解析的操作。

## 快速开始

### 创建连接

Bee 的连接需要提供一种 url 的参数形式, 其规则为 `${sess_mode}:${ds_mode}:${con_mode}:${uri}`, 其中各个参数说明如下: 

 - `sess_mode`: 执行脚本的类型， 比如当前选择 SQL 类型则该值为 : `sqlite`
 - `ds_mode`: 连接的数据源类型，比如当前选择 Agent 类型则该值为 : `agent`
 - `con_mod`: 连接到数据源的方式， 比如当前选择用户名/密码 的方式，则该值为 : `password`
 - `uri`: 连接到数据源时所需的 uri， 对于某些数据源，该值为必选项。该值必须符合 [URL Standard](http://url.spec.whatwg.org/)

```rust
use bee_core::Connection;

let conn = bee_core::new_connection(
    "sqlite:remote:password://oracle:test@127.0.0.1:22/bee?connect_timeout=5",
).unwrap();
```

### 执行请求

在 Bee 中执行一条脚本与 Java 的 JDBC 类似，只需输入脚本的内容和预计执行的时间即可。

```rust
use bee_core::Connection;

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
let statement = conn.new_statement(r#"
    SELECT  get(output,0,'TEXT','') as filesystem,
            get(output,1,'INT',0) as total,
            get(output,2,'INT',0) as used,
            get(output,3,'INT',0) as avail
    FROM (SELECT split_space(line) as output FROM remote_shell('df -k',10) 
    WHERE line NOT LIKE '%Filesystem%' AND line NOT LIKE '%tmp%')
"#, Duration::from_secs(4)).unwrap();

// 等待响应
let resp = statement.wait().unwrap();
let columns = resp.columns();
println!("columns - {:?}",columns);

// 这里是异步的过程
for rs in resp {
    let row = rs.unwrap();
    println!("row - {:?}",row);
}
```

## 结构模型概括

在 Bee 中主要提供了以下三种结构模型：

1. [脚本引擎](#脚本引擎): 一个连接中只可包含一个脚本引擎。脚本引擎用于解析和执行用户输入的脚本内容，并将其结果返回给用户。
2. [数据源](#数据源): 数据源是用于连接数据的地方，每个连接可拥有多个数据源。在脚本中使用数据源与普通函数调用类似，唯一不同的是，数据源返回的内容是异步多条的。
3. [扩展函数](#扩展函数): 可用于扩展脚本引擎内置函数的接口。扩展函数与数据源类似，唯一的区别在于扩展函数只能返回单条结果。

## 脚本引擎

目前 Bee 中主要实现了 SQL/Lua 两种类型的脚本支持。

### Lua

在 Bee 中 Lua 脚本引擎主要依赖 [rlua](https://github.com/amethyst/rlua) 项目来实现 Lua 语言的脚本引擎。

在 Lua 脚本引擎中内置了基本的数据源调用接口。比如解析 shell 命令 `df -k` 的输出结果，则可以采用以下 Lua 脚本:

```lua
-- 使用 remote_shell 数据源并执行 df -k 命令，超时时间为 10 s
local resp = remote_shell("df -k",10);

while(resp:has_next())
do
    local line = _next["line"];
    local line_num = _next["line_num"];
    -- 从第二行开始解析
    if(line_num > 0) then
        -- 使用空格作为切分符，将当前行切分为一个字符数组
        local cols = split_space(line);
        -- 按照规则来解析该字符数组
        _request:commit({
            filesystem  = get(cols,0,"TEXT",""), -- 解析 Filesystem 列作为 String 类型
            total       = get(cols,1,"INT",0),   -- 解析 1K-blocks 列作为 Integer 类型
            used        = get(cols,2,"INT",0),   -- 解析 Used 列作为 Integer 类型
            avail       = get(cols,3,"INT",0)    -- 解析 Available 列作为 Integer 类型
        });
    end
end
```

### Sqlite

在 Bee 中 SQL 语句采用 Sqlite 来实现的 SQL 语言的脚本引擎。 Bee 中的数据源被当作为 Sqlite 的 [vtab](https://www.sqlite.org/vtab.html)， 请参考官方的说明。

比如解析 shell 命令 `df -k` 的输出结果，则可以采用以下 SQL 脚本:

```sql
SELECT  get(output,0,'TEXT','') as filesystem, -- 解析 Filesystem 列作为 String 类型
        get(output,1,'INT',0) as total,        -- 解析 1K-blocks 列作为 Integer 类型
        get(output,2,'INT',0) as used,         -- 解析 Used 列作为 Integer 类型
        get(output,3,'INT',0) as avail         -- 解析 Available 列作为 Integer 类型
FROM (
    SELECT split_space(line) as output FROM remote_shell('df -k',10) WHERE line NOT LIKE '%Filesystem%' AND line NOT LIKE '%tmp%')
```

## 数据源

目前 Bee 中只支持了以下两种模式的数据源，可满足远程主机或本机监控数据的采集业务需求。

### Agent

在 Agent 数据源模式中提供了对本地的文件操作、基本的监控指标以及执行命令行的数据源类型。数据源列表如下: 

- [read_file](#read_file): 读取指定文件内容
- [mkdir](#mkdir): 创建目录
- [write_file](#write_file): 写入内容到指定文件中
- [shell](#shell): 执行命令
- [filesystem](#filesystem): 获取文件系统监控指标
- [host_basic](#host_basic): 获取主机基本信息
- [cpu_usage](#cpu_usage): 获取 CPU 使用率
- [os_info](#os_info): 获取操作系统基本信息
- [memory_usage](#memory_usage): 获取内存监控指标
- [swap_usage](#swap_usage): 获取 swap 监控指标

#### read_file

输入参数: 

1. 文件路径: String
2. 读取的开始位置: Integer
3. 读取的结束位置: Integer

输出结果行:

- `file_path`: 输入的文件路径(String)
- `file_size`: 文件大小(Integer)
- `content`: 读取到的文件内容(Bytes)

例如: 

```sql
SELECT * FROM read_file("/etc/hosts", 0, 4)
```

#### mkdir

输入参数: 

1. 目录路径: String

输出结果行:

- `success`: 执行成功返回 true 否则返回 false(Boolean)

例如: 

```sql
SELECT * FROM mkdir("/tmp")
```

#### write_file

输入参数: 

1. 文件路径: String
2. 写入的内容: String

输出结果行:

- `success`: 执行成功返回 true 否则返回 false(Boolean)

例如: 

```sql
SELECT * FROM write_file("/tmp/test", "Hello world")
```

#### shell

输入参数: 

1. 执行的命令: String
2. 执行的超时时间(s): Integer

输出结果行:

- `line`: 输出的字符行(String)
- `line_num`: 输出的字符行序号,从 0 开始(Integer)

例如: 

```sql
SELECT * FROM shell("echo Hello", 10) WHERE line_num = 0
```

#### filesystem

输入参数: 无

输出结果行:

- `name`: 文件系统名(String)
- `mount_on`: 挂载点(String)
- `total_bytes`: 总容量, bytes(Integer)
- `used_bytes`: 使用量, bytes(Integer)
- `free_bytes`: 可用量, bytes(Integer)

例如: 

```sql
SELECT * FROM filesystem() WHERE name NOT LIKE '%tmp%'
```

#### host_basic

输入参数: 无

输出结果行:

- `host_name`: 主机名(String)
- `cpu_core`: CPU 核心数(Integer)
- `cpu_model`: CPU 型号(String)
- `uptime`: 系统启动时间, 单位 s(Integer)
- `memory`: 内存容量, bytes(Integer)

例如: 

```sql
SELECT * FROM host_basic()
```

#### cpu_usage

输入参数: 无

输出结果行:

- `idle`: 空闲率(Number)
- `user`: 用户空间使用率(Number)
- `system`: 系统空间使用率(Number)
- `iowait`: (Number)

例如: 

```sql
SELECT * FROM cpu_usage()
```

#### os_info

输入参数: 无

输出结果行:

- `os_type`: 操作系统类型(String)
- `version`: 系统版本号(String)
- `host_name`: 主机名(String)

例如: 

```sql
 SELECT * FROM os_info()
```

#### memory_usage

输入参数: 无

输出结果行:

- `used_bytes`: 已使用量, bytes(String)
- `total_bytes`: 总容量, bytes(String)
- `free_bytes`: 可用量, bytes(String)

例如: 

```sql
 SELECT * FROM memory_usage()
```

#### swap_usage

输入参数: 无

输出结果行:

- `used_bytes`: 已使用量, bytes(String)
- `total_bytes`: 总容量, bytes(String)
- `free_bytes`: 可用量, bytes(String)

例如: 

```sql
 SELECT * FROM swap_usage()
```

### Remote

Remote 提供了一组操作远程主机的接口， 采用 [libssh](https://www.libssh.org/) 来实现 ssh 协议的支持。在 Remote 数据源模式中需要提供远程主机的连接信息，包括以下两种连接方式: 

- `password`: 用户名/密码
- `pubkey`: 用户名/公钥 

在 Remote 数据源模式中提供了对远程主机的文件操作、以及执行命令行的数据源类型。数据源列表如下: 

- [read_remote_file](#read_remote_file): 读取远程文件内容
- [upload_remote_file](#upload_remote_file): 上传本地文件到远程主机中
- [remote_mkdir](#remote_mkdir): 创建远程主机目录
- [remote_shell](#remote_shell): 执行远程命令

#### read_remote_file

输入参数: 

1. 文件路径: String
2. 读取的开始位置: Integer
3. 读取的结束位置: Integer

输出结果行:

- `file_path`: 输入的文件路径(String)
- `file_size`: 文件大小(Integer)
- `content`: 读取到的文件内容(Bytes)

例如: 

```sql
SELECT * FROM read_remote_file("/etc/hosts", 0, 4)
```

#### remote_mkdir

输入参数: 

1. 目录路径: String

输出结果行:

- `success`: 执行成功返回 true 否则返回 false(Boolean)

例如: 

```sql
SELECT * FROM remote_mkdir("/tmp")
```

#### upload_remote_file

输入参数: 

1. 文件路径: String
2. 写入的内容: String

输出结果行:

- `success`: 执行成功返回 true 否则返回 false(Boolean)

例如: 

```sql
SELECT * FROM upload_remote_file("/tmp/test", "Hello world")
```

#### remote_shell

输入参数: 

1. 执行的命令: String
2. 执行的超时时间(s): Integer

输出结果行:

- `line`: 输出的字符行(String)
- `line_num`: 输出的字符行序号,从 0 开始(Integer)

例如: 

```sql
SELECT * FROM remote_shell("echo Hello", 10) WHERE line_num = 0
```

## 扩展函数

在 Bee 中提供了以下扩展函数，方便实现输出结果的解析: 

- [get](#get): 提取并转换指定数组索引的值
- [split_csv](#split_csv): 使用 CSV 的格式来分隔字符串
- [split_space](#split_space): 使用空格来分隔字符串

### get

输入参数: 

1. 字符串数组的字节流(Bytes)
2. 数组索引
3. 将要转换的类型: (INT - 有符号64位 整型 | REAL - 有符号64位 浮点型 | TEXT - 字符串类型)(String)
4. 默认值，如果无法从该字符串数组中获取指定索引处的值，则使用该值转换返回

输出提取并转换后的结果。

### split_csv

输入参数: 

1. 待分隔的字符串 (String)

输出已分隔完成的字符串数组并采用 [bincode](https://github.com/servo/bincode) 转换为字节流，需要通过 [get](#get) 函数来获取其值。

### split_space

输入参数: 

1. 待分隔的字符串 (String)

输出已分隔完成的字符串数组并采用 [bincode](https://github.com/servo/bincode) 转换为字节流，需要通过 [get](#get) 函数来获取其值。