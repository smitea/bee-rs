use crate::{Error, Result, Value};
use std::convert::TryFrom;

/// 输入参数列表
#[derive(Debug, Clone)]
pub struct Args {
    values: Vec<Value>,
}

impl Args {
    /// 创建一个空的参数列表
    #[inline(always)]
    pub fn new() -> Self {
        Self { values: vec![] }
    }

    /// 获取参数值，通过参数索引
    pub fn get<T: TryFrom<Value, Error = Error>>(&self, index: usize) -> Result<T> {
        self.values
            .get(index)
            .map(|val| T::try_from(val.clone()))
            .ok_or(Error::index_range(index))?
    }

    /// 添加一个参数值
    #[inline(always)]
    pub fn push<T: Into<Value>>(&mut self, value: T) {
        self.values.push(value.into());
    }
}

impl Default for Args {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn test() {
    let args: Args = crate::args!(10, 20.0, "Name", false, vec![0x01, 0x02]);

    let val1 = args.get::<i64>(0).unwrap();
    assert_eq!(10, val1);

    let val1 = args.get::<f32>(1).unwrap();
    assert_eq!(20.0, val1);

    let val1 = args.get::<String>(2).unwrap();
    assert_eq!("Name".to_string(), val1);

    let val1 = args.get::<bool>(3).unwrap();
    assert_eq!(false, val1);

    let val1 = args.get::<Vec<u8>>(4).unwrap();
    assert_eq!(vec![0x01, 0x02], val1);

    let mut args: Args = Args::new();
    args.push(10);
    assert_eq!(10, args.get(0).unwrap());
    assert!(args.get::<f64>(0).is_err());
    assert!(args.get::<u32>(1).is_err());
}
