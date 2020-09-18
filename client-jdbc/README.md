# Bee 采集组件 *v0.0.1*

Bee 采集组件是基于 Scrape 项目之上开发的组件，目的是为了使 Scrape 支持 Windows 的采集业务，并且之后可替换 Scrape 采集功能，支持采集引擎的分布式扩展和单机部署。

- [依赖环境][#依赖环境]
- [快速开始](#快速开始)
- [设计概要](#设计概要)
  - [Plugin](#Plugin)
  - [Session](#Session)
- [可用插件](#可用插件)
  - [Lua](#Lua)
  - [Agent](#Agent)
  - [Mysql](#Mysql)
  - [Oracle](#Oracle)
  - [SSH](#SSH)
- [数据协议概要](#数据协议概要)
  - [数据类型](#数据类型)
  - [请求连接](#请求连接)
  - [数据表](#数据表)
  - [执行任务](#执行任务)
  - [关闭连接](#关闭连接)
  - [返回码](#返回码)
- [Changed](#Changed)

## 依赖环境

- [openssl](https://docs.rs/crate/openssl-sys/0.9.19) : 1.0.1 +
- [libssh](https://www.libssh.org/files/)  : 0.8.0 +
- [libcpuid](https://github.com/anrieff/libcpuid)

## 快速开始

```java
// 创建连接器
Connector connector = ConnectorFactory.defaultConnect("127.0.0.1", 6142, 10000);

// 获取 Session
Session session = connector.newSession("1", "agent", new Params());

// 执行采集请求
Response response = session.request("mounts").process(10L, TimeUnit.SECONDS);

// 需要自行 Ping 检测
new Thread(() -> {
    try {
        while (!connector.isClosed() && !Thread.interrupted()) {
            System.out.println("ping ...");
            // 发送 Ping 数据包
            connector.ping();
            System.out.println("ok ...");
            Thread.sleep(1000);
        }
    } catch (Exception e) {
        e.printStackTrace();
    }
}).start();

// 同步获取采集结果的响应流
while (response.hasNext()) {
    ResultRow row = response.next();
    System.out.println(row);
    Thread.sleep(1000);
}

// 一定需要关闭 Session 和 connect
session.close();
connector.close();
```

## 设计概要

Bee 延续了 Scrape 插件模式的设计思想，只是当前 Rust dynamic lib 在各个平台上的实现还不稳定，比如 Linux libc 2.5 < 的版本只支持 static lib。所以当前 Bee 还未实现插件物理结构的分离，但是你可以在编译时选择 Bee 的 features 来选用想要的插件。

### Plugin

Rust 语言提供了 Macro 支持，使得 Bee 插件的实现十分简单，比如提供一个方法，类似于 Session Factory:

```rust
pub fn plugin(_: Sessions, _: &Instance) -> Result<Box<dyn Session>, Error> {
    Ok(Box::new(AgentSession {
        statement: Arc::new(System::new()),
    }))
}
```

该方法用于创建 [Session](#Session)，该方法需要两个参数:

- `Sessions`: 当前 Bee 中已创建的所有 `Session`, 你可以通过该参数来获取需要关联的 `Session`
- `Instance`: 当前 `Session` 创建时所需要的连接信息

你可以在 Session 中定义你的包含的内容:

```rust
pub struct AgentSession {
    statement: Arc<System>,
}
```

然后开始定义你的 `Command` :

```rust
fn os_info(system: Arc<System>, request: &Request) -> Result<(), Error> {
    // 需要先发送 列表头 信息
    let commiter = request.head(head![
        ("os_type", STR),
        ("version", STR),
        ("bitness", STR),
        ("cpu_core", INTEGER),
        ("memory", LONG)
    ])?;
    let info = os_info::get();
    let cpu_core = num_cpus::get();
    let memory = system.memory()?;

    // 提交数据内容
    commiter.commit(row![
        info.os_type().to_string(),
        info.version().to_string(),
        info.bitness().to_string(),
        (cpu_core as u16),
        memory.total.as_u64()
    ])?;

    Ok(())
}
```

该方法用于创建 [Command](#Command)，该方法有两个参数:

- `Arc<System>`: 你所定义的 `Session` 中 Wrapper 的实例, 你可以使用它来实现当前 `Command` 所需的业务功能。比如该参数可以为 `Connection` 或者 `File` 等。
- `Request`: 该参数包含了请求的信息，并且你可以使用它来提交数据内容以及内容，类似 Vertx 中的 Handler 或者 Javascript 中的 Promise

下面可以采用 Macro 来把 [Command](#Command) 注册到 Session 中了:

```rust
session!((AgentSession,System,statement) => {
    "os_info"       => os_info
});
```

该 Macro 的第一个参数为你所定义的 Session 类型，第二个参数为该 Session 中的 Command 需要使用的属性类型，第三个参数为属性名称，该宏会自动帮你实现 `Session` 接口。`{}` 中的第一个参数为 `Command` 的名称，第二个为 `Command` 实现的方法。

注册插件需要提供 `PluginRegister` 实例，并且需要把你 `Session` 创建的方法给注册进去:

```rust
let mut register = PluginRegister::new();
// 注册一个名为 `agent` 的插件，该插件的实现在 `plugin` 方法中
register.register("agent", plugin);
```

完整的例子可参考 [测试用例](bee/src/lib.rs)

### Session

在 Bee 中, Session 是指一个目标端的表示形式，比如 `Oracle` 的一个连接实例可以表示为一个 Session。 Session 同时也是一组 [Command](#Command) 的集合，执行采集之前都必须要先创建一个 Session 之后才能进行之后的作业。

创建 Session 时需要提供一个 Instance 参数，该参数中需要提供该连接的 ID 和该连接所使用的插件名称，插件的使用可以参考 [Plugin](#Plugin), 比如:

```rust
let mut register = PluginRegister::new();
let instance = Instance::new("1", "agent");
register.register("agent", agent_plugin);
let bee = Bee::new(register, 4);
let sid = bee.accpet_instance(&instance).unwrap();
bee.accpet_request(&sid, "test", Params::new()).unwrap();
```

创建 Session 成功后，Bee 会返回一个 SID 用于之后执行采集请求的标示，该 SID 是全局唯一的。

## 可用插件

### Lua

Lua 插件并不提供采集功能，它只是负责执行 Lua 脚本，并且通过 Lua 脚本来调用其他采集插件提供的采集功能。

#### 请求方法

- `lua_script`: 执行参数中的脚本, 可用参数为:
  - `script`: Lua 脚本内容
- `lua_file`: 执行参数中的本地 Lua 文件， 可用参数为:
  - `file_path`: 本地 Lua 文件路径

#### 脚本 DSL

Lua 插件提供了部分 DSL 用于方便使用者快速调用其他插件提供的采集功能。 比如:

```lua
-- 使用 Agent 插件
session:use("agent")
-- 定义采集数据结构
local rq =
    session:headers(
    {
        avail = "String",
        fs_mounted_on = "String"
    }
)
-- 获取 OS 信息
local rs = agent.mounts()

while (rs:has_next()) do
    -- 获取结果行
    local row = rs:next()
    local avail = row["avail"]
    local fs_mounted_on = row["fs_mounted_on"]
    -- 提交采集数据
    rq:commit({avail, fs_mounted_on})
end
```

Lua 插件的实例信息需要提供一下连接参数:

- `ref_instances` (STR): 关联的实例 SID 列表，使用 `"sid1, sid2, sid3"` 的格式。

> Lua 插件的实例信息如果没有 `ref_instances` 参数，那么本身执行的 Lua 脚本就无法使用 `session:use` 函数，只能使用 `session:headers` & `rq:commit` 等基本的 DSL。

### Agent

Agent 插件提供了获取操作系统监控指标的采集功能

采集指标列表如下:

- `os_info`: 获取操作系统信息

    |filed| type | description|
    |--|--|--|
    |os_type| STR| 操作系统类型|
    |version| STR| 操作系统版本|
    |bitness| STR| 操作系统位数|
    |cpu_core| INTEGER| CPU 核心数|
    |memory| LONG| 内存大小|

- `mounts`: 获取磁盘信息

    |filed| type | description|
    |--|--|--|
    |free|LONG|可用空间大小|
    |avail|LONG|已用空间大小|
    |total|LONG|总共空间大小|
    |fs_type|STR|文件类型|
    |fs_mounted_from|STR|挂载路径|
    |fs_mounted_on|STR|挂载点|

- `cpu_usage`: 获取 CPU 使用率

    |filed| type | description|
    |--|--|--|
    |user|NUMBER| --|
    |nice|NUMBER| --|
    |system|NUMBER| --|
    |interrupt|NUMBER| --|
    |idle|NUMBER| --|

- `iops_usage`: 获取 IOPS

    |filed| type | description|
    |--|--|--|
    |name| STR| 磁盘名称|
    |read_ios| LONG|--|
    |read_merges|LONG|--|
    |read_sectors|LONG|--|
    |read_ticks|LONG|--|
    |write_ios|LONG|--|
    |write_merges|LONG|--|
    |write_sectors|LONG|--|
    |write_ticks|LONG|--|
    |in_flight|LONG|--|
    |io_ticks|LONG|--|
    |time_in_queue|LONG|--|

- `memory_usage`: 获取内存使用率

    |filed| type | description|
    |--|--|--|
    |used|LONG|已用内存大小|
    |total|LONG|总共内存大小|
    |free|LONG|可用内存大小|

- `loadavg`: 获取 CPU 平均负载

    |filed| type | description|
    |--|--|--|
    |one_m|NUMBER|一分钟内的平均负载|
    |five_m|NUMBER|五分钟内的平均负载|
    |fifteen_m|NUMBER|十五分钟内的平均负载|

- `command`: 执行操作系统命令
  - 参数:
    - `cmd` (STR): 待执行的命令
  - 返回结果:

    |filed| type | description|
    |--|--|--|
    |line|STR｜结果按行输出|

针对不同操作操作系统支持如下:

| API | MacOS | Window | Linux|
|--|--|--|--|
|os_info| x | ok | ok|
|mounts| ok | ok | ok|
|cpu_usage| x | ok | ok|
|iops_usage| x | x | ok|
|memory_usage| x | ok | ok|
|loadavg| x | x | ok|
|command| ok | ok | ok|

> 目前只支持 Window 2008(x64) + 版本, Linux(x64) 需要提供 libc 1.1.2 +  

### Mysql

TODO!

### Oracle

TODO!

### SSH

TODO!

## 数据协议概要

Bee 服务端在 SSH 协议之上重新开发了一套自己的数据传输协议，它具备一下特点:

- 伪全双工数据传输: 采集数据时可以异步返回采集内容， 但目前单个 Session 只支持单次数据采集。

- 连接复用: 一个连接可以建立多个 Session， 每个 Session 可以执行多次数据采集。

- 最小化数据传输: Bee 返回的数据格式采用 [数据表](#数据表) 的格式，由[表头](#数据头)(包含列名和数据类型)和[数据行](#数据行)(采集的单条数据)组成。表头在开始采集之前就已传输到 Client，之后只传输数据行内容，并且数据行内只存在采集数据信息，并未包含列名和数据类型的信息。

数据包协议组成格式:

| Head| Version | Type | Len | Data | CRC |END|
|--|--|--|--|--|--|--|
|4bit|1bit|1bit|8bit|...|8bit|2bit|
|0x00 0x42 0x65 0x65|0x01|...|...|...|...|0x0D 0x0A|

- Head: 固定位。共占用 4bit，固定值为 `Bee` (空格 + ASCII 码)
- Version: 版本号。目前为 1
- Type: 数据包类型，需要占用 1bit
  - `0`: 请求连接
  - `1`: 连接应答
  - `2`: 请求执行任务
  - `3`: 开始执行任务
  - `4`: 取消执行任务
  - `5`: 取消执行成功
  - `6`: Ping 请求
  - `7`: Ping 应答
  - `8`: 请求断开连接
- Len: 数据长度。需要占用 8bit，可表示为 64 位数字
- Data: 数据位
- CRC: 数据包总长度。需要占用 8bit，可表示为 64 位数字
- END: 固定位。共占用 2bit，固定值为 `\r\n` (ASCII 码)

### 数据类型

Bee 数据协议中，拥有一下基本的数据类型:

- `0`: String (字符串) 前 4 byte 表示为字符串数据位的长度，可以表示为 3GB 的字符串数据 (2^32)
- `1`: Number (双精度 64 位数字) 原始数据由 String 类型表示, 前 1 byte 表示为字符串数据位的长度
- `2`: Integer (有符号 64 位数字) 前 1 byte 表示为字符串数据位的长度
- `3`: Long (无符号 128 位数字) 原始数据由 String 类型表示, 前 1 byte 表示为字符串数据位的长度
- `4`: boolean (无符号 1 位数字) 前 1 byte 为固定长度 `1`，数据位为 `1` 时表示为 `true`, `0` 为 `false`
- `5`: Nill (无符号 1 为数字) 前 1 byte 为固定长度 `1`，无数据位

### 请求连接

- 请求:

  |client_id len | client_id | plugin_id len | plugin_id| params len | params |
  |--|--|--|--|--|--|
  |1bit|...|1bit|...|1bit|4bit|...|
  
  - client_id len: 客户端 ID 长度
  - client_id: 客户端 ID
  - plugin_id len: 插件 ID 长度
  - params len: 连接信息长度
  - params: 连接信息，可选

- 响应:

  |client_id len | client_id | plugin_id len | plugin_id| session_id len | session | code |
  |--|--|--|--|--|--|--|
  |1bit|...|1bit|...|1bit|1bit|...|1bit|

  - client_id len: 客户端 ID 长度。需要占用 1bit
  - client_id: 客户端 ID。
  - plugin_id len: 插件 ID 长度。需要占用 1bit
  - plugin_id: 插件 ID 长度。
  - session_id len: Session ID 长度。需要占用 1bit
  - session: Session ID
  - code: 返回码。需要占用 1bit

### 数据表

#### 数据头

|size|col1 type|col1 len|col1|col1 type|col2 len|col2|...|
|--|--|--|--|--|--|--|--|
|1bit|1bit|2bit|...|1bit|2bit|...|...|

- size: 列的总数。需要占用 1bit
- col1 type: 列的类型。需要占用 1bit
- col1 len: 列的长度。需要占用 1bit
- col1: 列名

从 `size` 之后的数据表示为数组的形式，直到解析完成

#### 数据行

|size|col1 type|col1 len|col1|col1 type|col2 len|col2|...|
|--|--|--|--|--|--|--|--|
|1bit|1bit|...|...|...|...|

- size: 列的总数。需要占用 1bit
- col1 type: 列数据的类型
  - `0`: String (字符串)
  - `1`: Number (双精度 64 位数字)
  - `2`: Integer (有符号 64 位数字)
  - `3`: Long (无符号 128 位数字)
  - `4`: boolean (无符号 1 位数字)
  - `5`: Nill (无符号 1 为数字)
- col1 len: 列数据的长度
  - `String`: 4
  - `Number`: 1
  - `Integer`: 1
  - `Long`: 1
  - `boolean`: 1
  - `Nill`: 1
- col1: 列数据

从 `size` 之后的数据表示为数组的形式，直到解析完成

### 执行任务

- 请求:

  | session_id len | sesssion | cmd_id len | cmd_id | params len | params|
  |--|--|--|--|--|--|
  | 1bit| ... | 1bit |...|4bit|...|
  
  - session_id len: Session ID 长度。需要占用 1bit
  - session: Session ID
  - cmd_id len: 任务 ID 长度。需要占用 1bit
  - cmd_id: 任务 ID
  - params len: 参数长度。需要占用 4bit
  - params: 任务参数, 可选

- 响应:

  |uid len | uid | session_id len | sesssion | cmd_id len | cmd_id | state | data len | data | code |code len | code msg|
  |--|--|--|--|--|--|--|--|--|--|--|--|
  |2bit| ... | 1bit| ... | 1bit |...|1bit | 4bit | ... |1bit|1bit|...|
  
  - uid len: Call UID 长度。需要占用 1bit
  - uid: Call UID
  - session_id len: Session ID 长度。需要占用 1bit
  - session: Session ID
  - cmd_id len: 任务 ID 长度。需要占用 1bit
  - cmd_id: 任务 ID
  - state: 任务状态码
    - `0`: 开始执行任务 (会响应数据格式信息, 会返回列表头部的信息)
    - `1`: 执行已执行 (会响应数据完整信息, 比如单条数据内容，列表的数据行内容)
    - `2`: 任务已完成 (会响应最终信息，比如执行失败或者失败)
  - data len: 数据位长度。需要占用 4bit
  - data: 请求任务响应数据内容
  - code: 返回码。需要占用 1bit
  - code len: 错误信息长度。需要占用 1bit
  - code msg: 错误信息

### 关闭连接

- 请求:

  |session_id len| sesssion|
  |--|--|
  |1bit| ... |

  - session_id len: Session ID 长度。需要占用 1bit
  - session: Session ID

- 响应:

  |session_id len | sesssion |
  |--|--|
  |1bit| ... |

  - session_id len: Session ID 长度。需要占用 1bit
  - session: Session ID

### 返回码

|type|code| msg len| msg
|--|--|--|--|
|1bit|4bit| 1bit | ... |

- type: 错误类型
- code: 返回码
- msg len: 错误信息长度
- msg: 错误信息

其中错误类型表示如下:

- `0x00`: 成功, 无 code & msg 信息
- `0x01`: IO 错误, 包括实例连接错误、连接超时、IO 读取错误等信息
- `0x03`: 参数错误
- `0x04`: 脚本错误
- `0x05`: 数据错误
- `0x06`: 其他错误

## Changed

- v0.0.1:
  - Basic framework
  - Lua & Agent plugin supported
  - Windows & Linux supported
  - Java SDK supported

## Docs
