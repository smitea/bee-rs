use crate::{Columns, Error, Row, Value};

#[derive(Debug, Clone)]
pub enum State {
    Ready(Columns),
    Process(Row),
    Err(Error),
    Ok,
}

pub trait ToData {
    fn columns() -> Columns;
    fn to_row(self) -> Row;
}

macro_rules! is_type {
    ($fun: ident : $variant:ident) => {
        #[inline(always)]
        pub fn $fun(&self) -> bool {
            return if let $crate::State::$variant = &self {
                true
            } else {
                false
            };
        }
    };

    ($fun: ident ,$variant:ident) => {
        #[inline(always)]
        pub fn $fun(&self) -> bool {
            return if let $crate::State::$variant(_) = &self {
                true
            } else {
                false
            };
        }
    };
}

impl State {
    is_type!(is_ok: Ok);
    is_type!(is_ready, Ready);
    is_type!(is_process, Process);
    is_type!(is_err, Err);

    #[inline(always)]
    pub fn from<T: Into<State>>(value: T) -> Self {
        value.into()
    }

    #[inline(always)]
    pub fn ok() -> Self {
        State::Ok
    }
}

impl From<Columns> for State {
    fn from(cols: Columns) -> Self {
        State::Ready(cols)
    }
}

impl From<Row> for State {
    fn from(row: Row) -> Self {
        State::Process(row)
    }
}

impl From<Error> for State {
    fn from(err: Error) -> Self {
        State::Err(err)
    }
}

impl From<Result<State, Error>> for State {
    fn from(rs: Result<State, Error>) -> Self {
        match rs {
            Ok(state) => state,
            Err(err) => State::Err(err),
        }
    }
}

#[test]
fn test() {
    let state = State::from(crate::columns![String: "Name", Number: "Age"]);
    assert!(state.is_ready());

    let state = State::from(crate::row!["Name", 20.0, 10, false, vec![0x01, 0x02], ()]);
    assert!(state.is_process());

    let state = State::from(Error::new(crate::error::OK, "Failed"));
    assert!(state.is_err());

    let state = State::ok();
    assert!(state.is_ok());
}
