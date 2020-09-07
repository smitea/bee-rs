use std::{fmt::Display, str::FromStr};
use url::ParseError;

#[macro_export]
macro_rules! code {
    ($base : expr, $index : expr) => {{
        $base | ($index << 8)
    }};
}

const OK: i32 = 0;

const INTERNAL: i32 = 2;
const INVALID: i32 = 3;
const PARAM: i32 = 4;
const CHANNEL: i32 = 6;
const IO: i32 = 7;
const LOCKED: i32 = 8;

const OTHER: i32 = 9;

const INVALID_TYPE: i32 = code!(INVALID, 1);
const INVALID_UTF8: i32 = code!(INVALID, 2);
const INVALID_URL: i32 = code!(INVALID, 3);
const INVALID_PATH: i32 = code!(INVALID, 4);

const PARAM_INDEX: i32 = code!(PARAM, 1);

const IO_NOTFOUND: i32 = code!(IO, 1);
const IO_PERMISSIONDENIED: i32 = code!(IO, 2);
const IO_CONNECTIONREFUSED: i32 = code!(IO, 3);
const IO_CONNECTIONRESET: i32 = code!(IO, 4);
const IO_CONNECTIONABORTED: i32 = code!(IO, 5);
const IO_NOTCONNECTED: i32 = code!(IO, 6);
const IO_ADDRINUSE: i32 = code!(IO, 7);
const IO_ADDRNOTAVAILABLE: i32 = code!(IO, 8);
const IO_BROKENPIPE: i32 = code!(IO, 9);
const IO_ALREADYEXISTS: i32 = code!(IO, 10);
const IO_INVALIDINPUT: i32 = code!(IO, 11);
const IO_INVALIDDATA: i32 = code!(IO, 12);
const IO_TIMEDOUT: i32 = code!(IO, 13);
const IO_WRITEZERO: i32 = code!(IO, 14);
const IO_OTHER: i32 = code!(IO, 15);
const IO_UNEXPECTEDEOF: i32 = code!(IO, 16);
const IO_WOULDBLOCK: i32 = code!(IO, 17);
const IO_INTERRUPTED: i32 = code!(IO, 18);
const CHANNEL_RECV: i32 = code!(CHANNEL, 1);
const CHANNEL_SEND: i32 = code!(CHANNEL, 2);

const MUTEX_LOCKED: i32 = code!(LOCKED, 1);

/// 错误信息(错误码和错误信息组成)
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Error {
    code: i32,
    msg: String,
}

pub type Result<T> = std::result::Result<T, Error>;

macro_rules! from_code {
    ($fun: ident ,$variant: expr, $T:ty) => {
        pub fn $fun(err: $T) -> Self {
            Self {
                code: $variant,
                msg: err.to_string(),
            }
        }
    };
}

impl Error {
    from_code!(invalid_type, INVALID_TYPE, String);
    from_code!(index_range, PARAM_INDEX, usize);
    from_code!(index_param, PARAM_INDEX, &str);
    from_code!(io_timeout, IO_TIMEDOUT, String);
    from_code!(lock_faild, LOCKED, &str);

    pub fn new<T: ToString>(code: i32, msg: T) -> Self {
        Self {
            code,
            msg: msg.to_string(),
        }
    }

    pub fn other<T: ToString>(code: i32, msg: T) -> Self {
        Self {
            code: (OTHER | (code << 8)),
            msg: msg.to_string(),
        }
    }

    pub fn internal<T: ToString>(code: i32, msg: T) -> Self {
        Self {
            code: (INTERNAL | (code << 8)),
            msg: msg.to_string(),
        }
    }

    pub fn invalid<T: ToString>(code: i32, msg: T) -> Self {
        Self {
            code: (INVALID | (code << 8)),
            msg: msg.to_string(),
        }
    }

    pub fn get_code(&self) -> i32 {
        *(&self.code)
    }

    pub fn get_msg(&self) -> &str {
        &self.msg
    }

    pub fn ok_code() -> i32 {
        OK
    }
}

macro_rules! from_error {
    ($variant: expr, $T:ty) => {
        impl From<$T> for Error {
            fn from(err: $T) -> Self {
                Error::new($variant, err.to_string())
            }
        }
    };
}

from_error!(CHANNEL_RECV, std::sync::mpsc::RecvTimeoutError);
from_error!(CHANNEL_RECV, std::sync::mpsc::RecvError);
impl<T> From<std::sync::mpsc::SendError<T>> for Error {
    fn from(err: std::sync::mpsc::SendError<T>) -> Self {
        Error::new(CHANNEL_SEND, err)
    }
}

from_error!(INVALID_UTF8, std::string::FromUtf8Error);
from_error!(INVALID_UTF8, std::str::Utf8Error);
from_error!(INVALID_PATH, std::convert::Infallible);
from_error!(INVALID_URL, ParseError);
from_error!(INVALID_TYPE, std::num::ParseIntError);
from_error!(INVALID_TYPE, std::num::ParseFloatError);

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Error::new(IO_NOTFOUND, err),
            std::io::ErrorKind::PermissionDenied => Error::new(IO_PERMISSIONDENIED, err),
            std::io::ErrorKind::ConnectionRefused => Error::new(IO_CONNECTIONREFUSED, err),
            std::io::ErrorKind::ConnectionReset => Error::new(IO_CONNECTIONRESET, err),
            std::io::ErrorKind::ConnectionAborted => Error::new(IO_CONNECTIONABORTED, err),
            std::io::ErrorKind::NotConnected => Error::new(IO_NOTCONNECTED, err),
            std::io::ErrorKind::AddrInUse => Error::new(IO_ADDRINUSE, err),
            std::io::ErrorKind::AddrNotAvailable => Error::new(IO_ADDRNOTAVAILABLE, err),
            std::io::ErrorKind::BrokenPipe => Error::new(IO_BROKENPIPE, err),
            std::io::ErrorKind::AlreadyExists => Error::new(IO_ALREADYEXISTS, err),
            std::io::ErrorKind::WouldBlock => Error::new(IO_WOULDBLOCK, err),
            std::io::ErrorKind::InvalidInput => Error::new(IO_INVALIDINPUT, err),
            std::io::ErrorKind::InvalidData => Error::new(IO_INVALIDDATA, err),
            std::io::ErrorKind::TimedOut => Error::new(IO_TIMEDOUT, err),
            std::io::ErrorKind::WriteZero => Error::new(IO_WRITEZERO, err),
            std::io::ErrorKind::Interrupted => Error::new(IO_INTERRUPTED, err),
            std::io::ErrorKind::UnexpectedEof => Error::new(IO_UNEXPECTEDEOF, err),
            _ => Error::new(IO_OTHER, err),
        }
    }
}

unsafe impl Send for Error {}

unsafe impl Sync for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.code, self.msg)
    }
}

impl FromStr for Error {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let values: Vec<&str> = s.split(",").collect();
        if let (Some(code), Some(msg)) = (values.get(0), values.get(1)) {
            let code: i32 = code.parse()?;
            Ok(Self {
                code,
                msg: msg.to_string(),
            })
        } else {
            Ok(Error::internal(OTHER, s))
        }
    }
}

impl std::error::Error for Error {}

impl<T> From<std::sync::PoisonError<std::sync::MutexGuard<'_, T>>> for Error {
    fn from(err: std::sync::PoisonError<std::sync::MutexGuard<'_, T>>) -> Self {
        Self {
            code: MUTEX_LOCKED,
            msg: err.to_string(),
        }
    }
}

impl<T> From<std::sync::PoisonError<std::sync::RwLockReadGuard<'_, T>>> for Error {
    fn from(err: std::sync::PoisonError<std::sync::RwLockReadGuard<'_, T>>) -> Self {
        Self {
            code: MUTEX_LOCKED,
            msg: err.to_string(),
        }
    }
}

impl<T> From<std::sync::PoisonError<parking_lot::ReentrantMutexGuard<'_, T>>> for Error {
    fn from(err: std::sync::PoisonError<parking_lot::ReentrantMutexGuard<'_, T>>) -> Self {
        Self {
            code: MUTEX_LOCKED,
            msg: err.to_string(),
        }
    }
}

#[test]
fn test() {
    let err = Error::other(1, "other error");
    assert_eq!(265, err.get_code());

    let err = Error::internal(1, "internal error");
    assert_eq!(258, err.get_code());

    let err = Error::invalid(1, "invalid error");
    assert_eq!(259, err.get_code());
    assert_eq!(OK, Error::ok_code());

    let err = std::sync::mpsc::RecvTimeoutError::Disconnected;
    assert_eq!(262, Error::from(err).get_code());

    let err = std::sync::mpsc::RecvError;
    assert_eq!(262, Error::from(err).get_code());

    if let Err(err) = String::from_utf8(vec![0, 159, 146, 150]) {
        assert_eq!(515, Error::from(err).get_code());
    }

    let err = std::sync::mpsc::SendError(10);
    assert_eq!(518, Error::from(err).get_code());

    let err = std::sync::mpsc::SendError(10);
    assert_eq!(518, Error::from(err).get_code());

    let err = std::io::Error::new(std::io::ErrorKind::NotFound, "failed");
    assert_eq!(263, Error::from(err).get_code());
    let err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "failed");
    assert_eq!(519, Error::from(err).get_code());
    let err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "failed");
    assert_eq!(775, Error::from(err).get_code());
    let err = std::io::Error::new(std::io::ErrorKind::ConnectionReset, "failed");
    assert_eq!(1031, Error::from(err).get_code());
    let err = std::io::Error::new(std::io::ErrorKind::ConnectionAborted, "failed");
    assert_eq!(1287, Error::from(err).get_code());
    let err = std::io::Error::new(std::io::ErrorKind::NotConnected, "failed");
    assert_eq!(1543, Error::from(err).get_code());
    let err = std::io::Error::new(std::io::ErrorKind::AddrInUse, "failed");
    assert_eq!(1799, Error::from(err).get_code());
    let err = std::io::Error::new(std::io::ErrorKind::AddrNotAvailable, "failed");
    assert_eq!(2055, Error::from(err).get_code());
    let err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "failed");
    assert_eq!(2311, Error::from(err).get_code());
    let err = std::io::Error::new(std::io::ErrorKind::AlreadyExists, "failed");
    assert_eq!(2567, Error::from(err).get_code());
    let err = std::io::Error::new(std::io::ErrorKind::WouldBlock, "failed");
    assert_eq!(4359, Error::from(err).get_code());
    let err = std::io::Error::new(std::io::ErrorKind::InvalidInput, "failed");
    assert_eq!(2823, Error::from(err).get_code());
    let err = std::io::Error::new(std::io::ErrorKind::InvalidData, "failed");
    assert_eq!(3079, Error::from(err).get_code());
    let err = std::io::Error::new(std::io::ErrorKind::TimedOut, "failed");
    assert_eq!(3335, Error::from(err).get_code());
    let err = std::io::Error::new(std::io::ErrorKind::WriteZero, "failed");
    assert_eq!(3591, Error::from(err).get_code());
    let err = std::io::Error::new(std::io::ErrorKind::Interrupted, "failed");
    assert_eq!(4615, Error::from(err).get_code());
    let err = std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "failed");
    assert_eq!(4103, Error::from(err).get_code());
    let err = std::io::Error::new(std::io::ErrorKind::Other, "failed");
    assert_eq!(3847, Error::from(err).get_code());

    let err = std::io::Error::new(std::io::ErrorKind::Other, "failed");
    assert_eq!("3847,failed".to_owned(), Error::from(err).to_string());

    let err: Error = "3847,failed".parse().unwrap();
    assert_eq!(3847, err.get_code());
    let err: Error = "failed".parse().unwrap();
    assert_eq!(2306, err.get_code());
}
