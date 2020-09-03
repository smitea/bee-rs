use crate::{DataType, Error};
use std::{convert::TryFrom, fmt::Display, str::FromStr};

pub type Bytes = Vec<u8>;

/// 所支持的值类型
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// 字符串类型
    String(String),
    /// 64 位有符号整型
    Integer(i64),
    /// 64 位有符号浮点型
    Number(f64),
    /// Boolean 类型
    Boolean(bool),
    /// 字节数组
    Bytes(Vec<u8>),
    /// 空值
    Nil,
}

impl Eq for Value {}

macro_rules! impl_into_value {
    ($variant:ident : $T:ty) => {
        impl From<$T> for Value {
            #[inline]
            fn from(val: $T) -> Value {
                Value::$variant(val.into())
            }
        }
    };
}

macro_rules! impl_try_from {
    ($variant:ident : $T:ty , $type_s: expr) => {
        impl TryFrom<Value> for $T {
            type Error = Error;
            fn try_from(value: Value) -> Result<Self, Self::Error> {
                if let Value::$variant(val) = value {
                    return Ok(val as $T);
                } else {
                    return Err(Error::invalid_type(format!(
                        "failed to parse {} for {:?}",
                        $type_s, value
                    )));
                }
            }
        }
    };
}

impl_into_value!(Integer: i64);
impl_into_value!(Integer: i32);
impl_into_value!(Integer: i16);
impl_into_value!(Integer: i8);
impl_into_value!(Integer: u32);
impl_into_value!(Integer: u16);
impl_into_value!(Integer: u8);
impl_into_value!(Number: f64);
impl_into_value!(Number: f32);
impl_into_value!(String: &str);
impl_into_value!(Boolean: bool);
impl_into_value!(Bytes: Bytes);

impl_try_from!(Integer: i64, "i64");
impl_try_from!(Integer: i32, "i32");
impl_try_from!(Integer: i16, "i16");
impl_try_from!(Integer: i8, "i8");
impl_try_from!(Integer: u32, "u32");
impl_try_from!(Integer: u16, "u16");
impl_try_from!(Integer: u8, "u8");
impl_try_from!(Number: f64, "f64");
impl_try_from!(Number: f32, "f32");
impl_try_from!(Boolean: bool, "bool");
impl_try_from!(Bytes: Bytes, "bytes");

impl Into<Value> for () {
    fn into(self) -> Value {
        Value::Nil
    }
}

impl TryFrom<Value> for String {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let val = match value {
            Value::String(val) => val,
            Value::Integer(val) => val.to_string(),
            Value::Number(val) => val.to_string(),
            Value::Boolean(val) => val.to_string(),
            Value::Bytes(val) => String::from_utf8(val)?,
            Value::Nil => "Nil".to_string(),
        };
        Ok(val)
    }
}

impl From<String> for Value {
    fn from(val: String) -> Self {
        Value::String(val)
    }
}

impl FromStr for Value {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = if s.contains("false") || s.contains("true") {
            s.parse::<bool>()
                .map(|v| Value::Boolean(v))
                .unwrap_or(Value::String(s.to_owned()))
        } else if s.contains(".") {
            s.parse::<f64>()
                .map(|v| Value::Number(v))
                .unwrap_or(Value::String(s.to_owned()))
        } else if s == "null" || s == "NULL" || s == "Null" {
            Value::Nil
        } else if s == "nil" || s == "Nil" {
            Value::Nil
        } else {
            s.parse::<i64>()
                .map(|v| Value::Integer(v))
                .unwrap_or(Value::String(s.to_owned()))
        };

        Ok(value)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Value::String(val) => write!(f, "{}", val),
            Value::Integer(val) => write!(f, "{}", val),
            Value::Number(val) => write!(f, "{}", val),
            Value::Boolean(val) => write!(f, "{}", val),
            Value::Bytes(val) => write!(f, "{:?}", val),
            Value::Nil => write!(f, "Nil"),
        }
    }
}

impl Value {
    pub fn get_type(&self) -> DataType {
        match self {
            Value::String(_) => DataType::String,
            Value::Integer(_) => DataType::Integer,
            Value::Number(_) => DataType::Number,
            Value::Boolean(_) => DataType::Boolean,
            Value::Bytes(_) => DataType::Bytes,
            Value::Nil => DataType::Nil,
        }
    }

    pub fn is_nil(&self) -> bool{
        if let Value::Nil = self{
            return true;
        }else{
            return false;
        }
    }
}

#[test]
fn from_test() {
    let val = "10".parse::<Value>().unwrap();
    assert_eq!(Value::Integer(10), val);

    let val = "false".parse::<Value>().unwrap();
    assert_eq!(Value::Boolean(false), val);

    let val = "true".parse::<Value>().unwrap();
    assert_eq!(Value::Boolean(true), val);

    let val = "10.01".parse::<Value>().unwrap();
    assert_eq!(Value::Number(10.01), val);

    let val = "10._".parse::<Value>().unwrap();
    assert_eq!(Value::String("10._".to_string()), val);

    let val = "Name".parse::<Value>().unwrap();
    assert_eq!(Value::String("Name".to_string()), val);

    let val = "Nil".parse::<Value>().unwrap();
    assert_eq!(Value::Nil, val);

    let val = "nil".parse::<Value>().unwrap();
    assert_eq!(Value::Nil, val);

    let val = "Null".parse::<Value>().unwrap();
    assert_eq!(Value::Nil, val);

    let val = "NULL".parse::<Value>().unwrap();
    assert_eq!(Value::Nil, val);

    let val = "null".parse::<Value>().unwrap();
    assert_eq!(Value::Nil, val);
}

#[test]
fn from_value() {
    let val: Value = 10.into();
    assert_eq!(Value::Integer(10), val);

    let val: Value = 10.0.into();
    assert_eq!(Value::Number(10.0), val);

    let val: Value = false.into();
    assert_eq!(Value::Boolean(false), val);

    let val: Value = "Name".into();
    assert_eq!(Value::String("Name".to_owned()), val);

    let val: Value = ().into();
    assert_eq!(Value::Nil, val);

    let val: Value = vec![0x09_u8, 0x12].into();
    assert_eq!(Value::Bytes(vec![0x09_u8, 0x12]), val);
}

#[test]
fn try_from_value(){
    assert_eq!(Value::Number(10.0), Value::try_from(10.0).unwrap());
    assert_eq!(Value::Boolean(false), Value::try_from(false).unwrap());
    assert_eq!(Value::Integer(10), Value::try_from(10).unwrap());
    assert_eq!(Value::String("He".to_owned()), Value::try_from("He".to_owned()).unwrap());
    assert_eq!(Value::Bytes(b"\x00\x01".to_vec()), Value::try_from(b"\x00\x01".to_vec()).unwrap());
    assert_eq!(Value::Nil, Value::try_from(()).unwrap());
}

#[test]
#[should_panic(expected = "failed to parse")]
fn try_from_value_failed(){
    let _: u8 = u8::try_from(Value::Number(10.02)).unwrap();
}

#[test]
fn get_type(){
    assert_eq!(DataType::Number, Value::from(10.0).get_type());
    assert_eq!(DataType::Boolean, Value::from(false).get_type());
    assert_eq!(DataType::Integer, Value::from(10).get_type());
    assert_eq!(DataType::String, Value::from("He".to_owned()).get_type());
    assert_eq!(DataType::Bytes, Value::from(b"\x00\x01".to_vec()).get_type());
    assert_eq!(DataType::Nil, Value::Nil.get_type());
    assert!(Value::Nil.is_nil());
    assert!(!Value::Number(10.0).is_nil());
}

#[test]
fn to_string(){
    assert_eq!("10.02".to_string(), format!("{}",Value::from(10.02_f64)));
    assert_eq!("10".to_string(), format!("{}",Value::from(10)));
    assert_eq!("false".to_string(), format!("{}",Value::from(false)));
    assert_eq!("He".to_string(), format!("{}",Value::from("He".to_owned())));
    assert_eq!("[0, 1]".to_string(), format!("{}",Value::from(b"\x00\x01".to_vec())));
    assert_eq!("Nil",format!("{}",Value::Nil));
}