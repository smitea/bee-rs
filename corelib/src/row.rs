use crate::{Error, Value};
use std::convert::TryFrom;

#[derive(Clone,Debug)]
pub struct Row {
    pub(crate) values: Vec<Value>,
}

impl Row {
    #[inline(always)]
    pub fn new() -> Self {
        Row { values: vec![] }
    }

    pub fn get<T: TryFrom<Value, Error = Error>>(&self, index: usize) -> Result<T, Error> {
        self.values
            .get(index)
            .map(|val| T::try_from(val.clone()))
            .ok_or(Error::index_range(index))?
    }

    pub fn get_value(&self, index: usize) -> Result<&Value, Error> {
        self.values
            .get(index)
            .map(|val| Ok(val))
            .ok_or(Error::index_range(index))?
    }

    #[inline(always)]
    pub fn push<T: Into<Value>>(&mut self, value: T) {
        self.values.push(value.into())
    }
}

impl Default for Row {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! row {
    ($($val: expr),*)=> {
        {
            let mut row = $crate::Row::new();
            $(
                row.push($val);
            )*
            row
        }
    };
}

#[test]
fn test() {
    let row: Row = row!(10, 20.0, "Name", false, vec![0x01, 0x02]);

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
