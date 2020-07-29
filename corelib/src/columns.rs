use crate::DataType;
use std::ops::Deref;

#[derive(Debug, Clone,Eq, PartialEq)]
pub struct Columns {
    pub(crate) values: Vec<(String, DataType)>,
}

impl Columns {
    #[inline(always)]
    pub fn new() -> Self {
        Self { values: vec![] }
    }

    pub fn get_name(&self, index: usize) -> Option<&String> {
        self.values.get(index).map(|val| &val.0)
    }

    pub fn get_index<T: Into<String>>(&self, name: T) -> Option<usize> {
        let name = name.into();
        for (index, val) in self.values.iter().enumerate() {
            if name == val.0 {
                return Some(index);
            }
        }
        return None;
    }

    #[inline(always)]
    pub fn push<K: Into<String>>(&mut self, key: K, value: DataType) {
        self.values.push((key.into(), value));
    }
}

impl Deref for Columns {
    type Target = Vec<(String, DataType)>;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl AsRef<Vec<(String, DataType)>> for Columns {
    fn as_ref(&self) -> &Vec<(String, DataType)> {
        &self.values
    }
}

impl Default for Columns {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn test() {
    let columns = crate::columns![
        String  : "Name",
        Number  : "Age",
        Integer : "Count",
        Boolean : "IsNice",
        Bytes   : "Image",
        Nil     : "Phone"
    ];

    assert_eq!(Some(0), columns.get_index("Name"));
    assert_eq!(Some(1), columns.get_index("Age"));
    assert_eq!(Some(2), columns.get_index("Count"));
    assert_eq!(Some(3), columns.get_index("IsNice"));
    assert_eq!(Some(4), columns.get_index("Image"));
    assert_eq!(Some(5), columns.get_index("Phone"));
}
