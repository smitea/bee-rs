use crate::{Args, Columns, Error, Request, Row, State};
use std::{
    sync::mpsc::{sync_channel, Receiver, RecvTimeoutError},
    time::Duration,
};

const CHANNEL_SIZE: usize = 1024;

pub struct Statement {
    tx: Receiver<State>,
    timeout: Option<Duration>,
}

pub struct Response {
    timeout: Option<Duration>,
    columns: Columns,
    tx: Receiver<State>,
}

impl Response {
    #[inline(always)]
    pub fn columns(&self) -> &Columns {
        &self.columns
    }
}

impl Statement {
    #[inline(always)]
    pub fn new(timeout: Option<Duration>, tx: Receiver<State>) -> Self {
        Self { tx, timeout }
    }

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
        
        return Err(Error::from(std::io::Error::from(
            std::io::ErrorKind::InvalidData,
        )));
    }
}

impl Response {
    pub fn next_row(&self) -> Option<Result<Row, Error>> {
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

pub fn new_req(args: Args, timeout: Duration) -> (Request, Statement) {
    let (tx, rx) = sync_channel(CHANNEL_SIZE);
    let request = Request::new(args, tx);
    let statement = Statement::new(Some(timeout), rx);
    return (request, statement);
}

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
    std::thread::spawn(move || {
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
