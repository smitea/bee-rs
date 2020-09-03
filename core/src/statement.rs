use crate::{Args, Columns, Error, Request, Row, State};
use smol::channel::*;

const MAX_CAP: usize = 1024;

/// 请求执行后的结果集
pub struct Statement {
    /// 数据流接收器
    tx: Receiver<State>,
}

/// 请求执行后的响应内容
pub struct Response {
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
    pub fn new(tx: Receiver<State>) -> Self {
        Self { tx }
    }

    /// 等待数据响应，返回响应内容
    pub async fn wait(self) -> Result<Response, Error> {
        let state = self.tx.recv().await?;

        if let State::Ready(columns) = state {
            return Ok(Response {
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
    pub async fn next_row(&self) -> Option<Result<Row, Error>> {
        match self.tx.recv().await {
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

impl Iterator for Response {
    type Item = Result<Row, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        smol::block_on(async { self.next_row().await })
    }
}

/// 创建一个请求和一个结果集，通过请求参数列表 `args` 和 最大执行时间 `timeout`
pub fn new_req(args: Args) -> (Request, Statement) {
    let (tx, rx) = smol::channel::bounded(MAX_CAP);
    let request = Request::new(args, tx);
    let statement = Statement::new(rx);
    return (request, statement);
}

#[test]
fn test() {
    use crate::Args;
    use std::time::Duration;

    smol::block_on(async {
        let (request, statement) = new_req(Args::new());
        let _ = smol::block_on(async move {
            let mut promise = request
                .new_commit(crate::columns![String: "name", Number: "age", Integer: "row_id", Boolean: "is_new", Bytes: "image"])
                .await
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

        let resp = statement.wait().await.unwrap();
        let columns = resp.columns();

        assert_eq!(Some(0), columns.get_index("name"));
        assert!(resp.into_iter().all(|x| x.is_ok()));
    });
}