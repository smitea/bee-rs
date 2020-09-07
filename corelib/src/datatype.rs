use crate::{value::Bytes, Error, Value};
use std::{fmt::Display, str::FromStr};

/// 数据类型
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DataType {
    String,
    Integer,
    Number,
    Boolean,
    Bytes,
    Nil,
}

/// 将其他类型转行为当前的数据类型实例
pub trait ToType {
    fn get_type() -> DataType;
}

macro_rules! impl_to_type {
    ($t1: ty, $t2: ident) => {
        impl $crate::ToType for $t1 {
            fn get_type() -> $crate::DataType {
                $crate::DataType::$t2
            }
        }
    };
}

impl_to_type!(i64, Integer);
impl_to_type!(i32, Integer);
impl_to_type!(i16, Integer);
impl_to_type!(i8, Integer);
impl_to_type!(u32, Integer);
impl_to_type!(u16, Integer);
impl_to_type!(u8, Integer);
impl_to_type!(f64, Number);
impl_to_type!(f32, Number);
impl_to_type!(bool, Boolean);
impl_to_type!(String, String);
impl_to_type!(Bytes, Bytes);
impl_to_type!([u8], Bytes);
impl_to_type!((), Nil);

impl FromStr for DataType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data_type: &str = &s.to_lowercase();
        let t = match data_type {
            "string" => DataType::String,
            "integer" => DataType::Integer,
            "i64" => DataType::Integer,
            "i32" => DataType::Integer,
            "i16" => DataType::Integer,
            "i8" => DataType::Integer,
            "u32" => DataType::Integer,
            "u16" => DataType::Integer,
            "u8" => DataType::Integer,

            "number" => DataType::Number,
            "f64" => DataType::Number,
            "f32" => DataType::Number,
            "boolean" => DataType::Boolean,
            "bytes" => DataType::Bytes,
            "vec<u8>" => DataType::Bytes,
            "()" => DataType::Nil,
            "null" => DataType::Nil,
            "nil" => DataType::Nil,
            _ => {
                return Err(Error::invalid_type(format!(
                    "failed to parse str {} for DataType",
                    s
                )))
            }
        };

        return Ok(t);
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::String => write!(f, "String"),
            DataType::Integer => write!(f, "Integer"),
            DataType::Number => write!(f, "Number"),
            DataType::Boolean => write!(f, "Boolean"),
            DataType::Bytes => write!(f, "Bytes"),
            DataType::Nil => write!(f, "Nil"),
        }
    }
}

impl From<Value> for DataType {
    fn from(val: Value) -> Self {
        match val {
            Value::String(_) => DataType::String,
            Value::Integer(_) => DataType::Integer,
            Value::Number(_) => DataType::Number,
            Value::Boolean(_) => DataType::Boolean,
            Value::Bytes(_) => DataType::Bytes,
            Value::Nil => DataType::Nil,
        }
    }
}

#[test]
fn test() {
    let t: DataType = "String".parse().unwrap();
    assert_eq!("String".to_owned(), t.to_string());

    let t: DataType = "Integer".parse().unwrap();
    assert_eq!("Integer".to_owned(), t.to_string());
    let t: DataType = "i64".parse().unwrap();
    assert_eq!("Integer".to_owned(), t.to_string());
    let t: DataType = "i32".parse().unwrap();
    assert_eq!("Integer".to_owned(), t.to_string());
    let t: DataType = "i16".parse().unwrap();
    assert_eq!("Integer".to_owned(), t.to_string());
    let t: DataType = "i8".parse().unwrap();
    assert_eq!("Integer".to_owned(), t.to_string());
    let t: DataType = "u32".parse().unwrap();
    assert_eq!("Integer".to_owned(), t.to_string());
    let t: DataType = "u16".parse().unwrap();
    assert_eq!("Integer".to_owned(), t.to_string());
    let t: DataType = "u8".parse().unwrap();
    assert_eq!("Integer".to_owned(), t.to_string());

    let t: DataType = "Number".parse().unwrap();
    assert_eq!("Number".to_owned(), t.to_string());
    let t: DataType = "f64".parse().unwrap();
    assert_eq!("Number".to_owned(), t.to_string());
    let t: DataType = "f32".parse().unwrap();
    assert_eq!("Number".to_owned(), t.to_string());

    let t: DataType = "Boolean".parse().unwrap();
    assert_eq!("Boolean".to_owned(), t.to_string());

    let t: DataType = "Bytes".parse().unwrap();
    assert_eq!("Bytes".to_owned(), t.to_string());
    let t: DataType = "vec<u8>".parse().unwrap();
    assert_eq!("Bytes".to_owned(), t.to_string());

    let t: DataType = "()".parse().unwrap();
    assert_eq!("Nil".to_owned(), t.to_string());
    let t: DataType = "Nil".parse().unwrap();
    assert_eq!("Nil".to_owned(), t.to_string());

    let t: crate::Result<DataType> = "test".parse();
    assert!(t.is_err());

    assert_eq!(DataType::Boolean, DataType::from(Value::from(false)));
    assert_eq!(DataType::Number, DataType::from(Value::from(10.02)));
    assert_eq!(DataType::Integer, DataType::from(Value::from(10)));
    assert_eq!(
        DataType::String,
        DataType::from(Value::from("name".to_owned()))
    );
    assert_eq!(
        DataType::Bytes,
        DataType::from(Value::from(b"\x01\x02".to_vec()))
    );
    assert_eq!(DataType::Nil, DataType::from(Value::Nil));
}
