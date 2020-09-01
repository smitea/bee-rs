use crate::{Args, Columns, Error, Request, Row, State};
use std::{
    sync::mpsc::{sync_channel, Receiver, RecvTimeoutError},
    time::Duration,
};

const CHANNEL_SIZE: usize = 2;

/// 请求执行后的结果集
pub struct Statement {
    /// 数据流接收器
    tx: Receiver<State>,
    /// 最大执行时间
    timeout: Option<Duration>,
}

/// 请求执行后的响应内容
pub struct Response {
    /// 最大执行时间
    timeout: Option<Duration>,
    /// 列结构的定义
    columns: Columns,
    /// 数据流接收器
    tx: Receiver<State>,
}

impl Response {
    /// 获取列结构的定义
    #[inline(always)]
    pub fn columns(&self) -> &Columns {
        &self.columns
    }
}

impl Drop for Response {
    fn drop(&mut self) {
        drop(&self.tx);
        drop(&self.columns);
    }
}

impl Statement {
    /// 创建结果集，通过最大执行时间 `timeout` 和 数据流接收器 `tx`
    #[inline(always)]
    pub fn new(timeout: Option<Duration>, tx: Receiver<State>) -> Self {
        Self { tx, timeout }
    }

    /// 等待数据响应，返回响应内容
    pub fn wait(self) -> Result<Response, Error> {
        let state = if let Some(timeout) = self.timeout {
            self.tx.recv_timeout(timeout)?
        } else {
            self.tx.recv()?
        };

        if let State::Ready(columns) = state {
            return Ok(Response {
                timeout: self.timeout,
                columns,
                tx: self.tx,
            });
        } else if let State::Err(err) = state {
            return Err(err);
        }
        return Err(Error::invalid_type(format!(
            "invalid to wait a columns for response"
        )));
    }
}

impl Response {
    /// 获取下一个数据行内容
    pub fn next_row(&self) -> Option<Result<Row, Error>> {
        // 如果设置了最大执行时间，则需要超时机制保证
        if let Some(timeout) = self.timeout {
            match self.tx.recv_timeout(timeout) {
                Ok(state) => match state {
                    State::Process(row) => {
                        return Some(Ok(row));
                    }
                    State::Err(err) => return Some(Err(err)),
                    _ => {
                        return None;
                    }
                },
                Err(RecvTimeoutError::Disconnected) => return None,
                Err(err) => return Some(Err(Error::from(err))),
            }
        } else {
            match self.tx.recv() {
                Ok(state) => match state {
                    State::Process(row) => {
                        return Some(Ok(row));
                    }
                    State::Err(err) => return Some(Err(err)),
                    _ => {
                        return None;
                    }
                },
                Err(err) => return Some(Err(Error::from(err))),
            }
        }
    }
}

impl Iterator for Response {
    type Item = Result<Row, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_row()
    }
}

/// 创建一个请求和一个结果集，通过请求参数列表 `args` 和 最大执行时间 `timeout`
pub fn new_req(args: Args, timeout: Duration) -> (Request, Statement) {
    let (tx, rx) = sync_channel(CHANNEL_SIZE);
    let request = Request::new(args, tx);
    let statement = Statement::new(Some(timeout), rx);
    return (request, statement);
}

/// 创建一个请求和一个结果集，通过请求参数列表 `args`, 无最大执行时间
pub fn new_req_none(args: Args) -> (Request, Statement) {
    let (tx, rx) = sync_channel(CHANNEL_SIZE);
    let request = Request::new(args, tx);
    let statement = Statement::new(None, rx);
    return (request, statement);
}

#[test]
fn test() {
    use crate::{Args, Request};
    use std::sync::mpsc::*;

    let (rx, tx) = sync_channel::<State>(1024);
    let request = Request::new(Args::new(), rx);
    let _ = std::thread::spawn(move || {
        let mut promise = request
            .new_commit(crate::columns![String: "name", Number: "age", Integer: "row_id", Boolean: "is_new", Bytes: "image"])
            .unwrap();

        std::thread::sleep(Duration::from_millis(100));
        let _ = promise.commit(State::from(crate::row![
            "He",
            20.0,
            10,
            false,
            vec![0x01, 0x02]
        ]));
        let _ = promise.commit(State::Ok);
    });

    let statement = Statement::new(Some(Duration::from_secs(1)), tx);
    let resp = statement.wait().unwrap();
    let columns = resp.columns();

    assert_eq!(Some(0), columns.get_index("name"));
    assert!(resp.into_iter().all(|x| x.is_ok()));
}

#[test]
fn test_without_timeout() {
    let (mut req, statement) = new_req_none(crate::args![10, 10.02]);
    let _ = std::thread::spawn(move || {
        let args = req.get_args();
        let arg0: u32 = args.get(0).unwrap();
        assert_eq!(10, arg0);

        let commit = req.new_commit(crate::columns![String: "name"]).unwrap();
        let args = commit.get_args();
        let arg0: f64 = args.get(1).unwrap();
        assert_eq!(10.02, arg0);
        std::thread::sleep(Duration::from_millis(100));
        if let Err(err) = req.commit(vec![("age".to_owned(), crate::Value::from(10))]) {
            req.error(err).unwrap();
        }
    });

    let resp = statement.wait().unwrap();
    assert_eq!(&crate::columns![String: "name"], resp.columns());

    for row in resp {
        let _ = row.unwrap();
    }
}
