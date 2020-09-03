use crate::DataType;
use std::ops::Deref;

/// 数据列结构定义
#[derive(Debug, Clone,Eq, PartialEq)]
pub struct Columns {
    pub(crate) values: Vec<(String, DataType)>,
}

impl Columns {
    /// 创建一个空的结构
    #[inline(always)]
    pub fn new() -> Self {
        Self { values: vec![] }
    }

    /// 获取列名通过列索引
    pub fn get_name(&self, index: usize) -> Option<&String> {
        self.values.get(index).map(|val| &val.0)
    }

    /// 获取列索引通过列名
    pub fn get_index<T: Into<String>>(&self, name: T) -> Option<usize> {
        let name = name.into();
        for (index, val) in self.values.iter().enumerate() {
            if name == val.0 {
                return Some(index);
            }
        }
        return None;
    }

    /// 添加列名的结构定义
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

    let mut columns = crate::Columns::new();
    columns.push("name", DataType::String);

    assert_eq!(Option::Some(&"name".to_owned()),columns.get_name(0));
    assert_eq!(Option::None,columns.get_name(1));

    assert_eq!(Option::Some(0),columns.get_index("name"));
    assert_eq!(Option::None,columns.get_index("age"));

    assert!(columns.iter().len() > 0);
}
