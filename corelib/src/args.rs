use crate::{Error, Value};
use std::convert::TryFrom;

#[derive(Debug,Clone)]
pub struct Args {
    values: Vec<Value>,
}

impl Args {

    #[inline(always)]
    pub fn new() -> Self {
        Self { values: vec![] }
    }

    pub fn get<T: TryFrom<Value, Error = Error>>(&self, index: usize) -> Result<T, Error> {
        self.values
            .get(index)
            .map(|val| T::try_from(val.clone()))
            .ok_or(Error::index_range(index))?
    }

    #[inline(always)]
    pub fn push<T: Into<Value>>(&mut self, value: T){
        self.values.push(value.into());
    }
}

impl Default for Args {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! args {
    ($($val: expr),*) => {{
        let mut args: $crate::Args = $crate::Args::new();
        $(
            args.push($val);
        )*

        args
    }};
}

#[test]
fn test() {
    let args: Args = args!(10, 20.0, "Name", false, vec![0x01, 0x02]);

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
}
