use crate::{Error, Result, Value};
use std::{convert::TryFrom, ops::Deref};

/// 数据行
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub(crate) values: Vec<Value>,
}

impl Row {
    /// 创建一个数据行
    #[inline(always)]
    pub fn new() -> Self {
        Row { values: vec![] }
    }

    /// 获取数据行的内容(该内容的类型可以通过 `T` 来确定)，通过指定的索引，如果获取失败则返回错误
    pub fn get<T: TryFrom<Value, Error = Error>>(&self, index: usize) -> Result<T> {
        self.values
            .get(index)
            .map(|val| T::try_from(val.clone()))
            .ok_or(Error::index_range(index))?
    }

    /// 获取数据行的内容，通过指定的索引，如果获取失败则返回错误
    pub fn get_value(&self, index: usize) -> Result<&Value> {
        self.values
            .get(index)
            .map(|val| Ok(val))
            .ok_or(Error::index_range(index))?
    }

    /// 添加数据列到该数据行中，值为 `T` 类型
    #[inline(always)]
    pub fn push<T: Into<Value>>(&mut self, value: T) {
        self.values.push(value.into())
    }
}

impl Deref for Row {
    type Target = Vec<Value>;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl AsRef<Vec<Value>> for Row {
    fn as_ref(&self) -> &Vec<Value> {
        &self.values
    }
}

impl Default for Row {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn test() {
    let row: Row = crate::row!(10, 20.0, "Name", false, vec![0x01, 0x02]);

    let val1 = row.get::<i64>(0).unwrap();
    assert_eq!(10, val1);

    let val1 = row.get::<f32>(1).unwrap();
    assert_eq!(20.0, val1);

    let val1 = row.get::<String>(2).unwrap();
    assert_eq!("Name".to_string(), val1);

    let val1 = row.get::<bool>(3).unwrap();
    assert_eq!(false, val1);

    let val1 = row.get::<Vec<u8>>(4).unwrap();
    assert_eq!(vec![0x01, 0x02], val1);
}

#[test]
fn test_faild() {
    let row: Row = crate::row!(10, 20.0, "Name", false, vec![0x01, 0x02]);
    assert!(row.get_value(5).is_err());

    assert!(row.get::<f64>(0).is_err());
    assert!(row.get::<u32>(5).is_err());
    assert!(row.get_value(5).is_err());

    let mut row: Row = Row::default();
    row.push("He");
    assert!(row.get_value(0).is_ok());

    assert_eq!(1, row.iter().len());
}
