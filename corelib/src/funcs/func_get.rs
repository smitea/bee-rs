use crate::{Error, Result, Value};

#[function]
pub fn get(output: Vec<u8>, index: i32, data_type: String, default: String) -> Result<Value> {
    let array: Vec<String> =
        bincode::deserialize(&output).or_else(|err| Err(Error::other(0, format!("{}", err))))?;

    let len = array.len() as i32;
    let mut index = index;
    if index < 0 {
        index = len + index;
    }
    let value = array.get(index as usize).unwrap_or(&default);
    let data_type = data_type.as_str();

    parse_value(data_type, value)
}

fn parse_value(data_type: &str, value: &String) -> Result<Value> {
    debug!("parser_value : {} - {}", data_type, value);
    let value = match data_type {
        "INT" => {
            let value = value.parse::<i64>()?;
            Value::Integer(value)
        }
        "REAL" => {
            let value = value.parse::<f64>()?;
            Value::Number(value)
        }
        _ => Value::String(value.clone()),
    };

    Ok(value)
}

#[test]
fn test() {
    use crate::*;
    let record = vec!["10", "10.02", "He"];
    let bytes = bincode::serialize(&record).unwrap();
    assert_eq!(
        Value::from(10),
        get(bytes.clone(), 0, "INT".to_string(), "0".to_string()).unwrap()
    );
    assert_eq!(
        Value::from(10.02),
        get(bytes.clone(), 1, "REAL".to_string(), "0".to_string()).unwrap()
    );
    assert_eq!(
        Value::from("He"),
        get(bytes.clone(), 2, "TEXT".to_string(), "0".to_string()).unwrap()
    );

    assert_eq!(
        Value::from("He"),
        get(bytes, -1, "TEXT".to_string(), "He".to_string()).unwrap()
    );
}
