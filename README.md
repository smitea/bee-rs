# Bee 

[![Build Status](https://travis-ci.org/sMitea/bee-rs.svg?branch=master)](https://travis-ci.org/sMitea/bee-rs)
[![codecov](https://codecov.io/gh/sMitea/bee-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/sMitea/bee-rs)

Bee 采用一种基于脚本的数据解析方式，可使用 Lua/SQL 来对主机监控数据爬取和解析的操作。

- [corelib](corelib/README.md): Bee 的实现
- [client](client/README.md): 脚本测试工具
- [client-jdbc](client-jdbc/README.md): JDBC 驱动的实现
- [codec](codec/README.md): 数据传输协议的实现
- [server](server/README.md): 数据传输协议的服务端实现

## Cross Complie

需要使用 Docker 来提供交叉编译环境，当前交叉编译工具使用的是 [rust-cross](https://github.com/rust-embedded/cross)。 下面列出比较常用的操作系统的编译命令:

- Unbuntu(> 14.0): `./cross-build.sh x86_64-unknown-linux-gnu`
- Windows(>= 2008R): `./cross-build.sh x86_64-pc-windows-gnu`

> 暂时不支持 musl 交叉编译，未能解决 openssl-sys 在 musl-gcc 下的库函数与 libssh 不兼容的问题。
