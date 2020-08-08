use crate::{Error, Result, Value};

#[function]
pub fn get(output: Vec<u8>, index: i32, data_type: String, default: String) -> Result<Value> {
    let array: Vec<String> =
        bincode::deserialize(&output).or_else(|err| Err(Error::other(0, format!("{}", err))))?;

    let len = array.len() as i32;
    let mut index = index;
    if index < 0 {
        index = len + index - 1;
    }
    let value = array.get(index as usize).unwrap_or(&default);
    let data_type = data_type.as_str();

    parse_value(data_type, value)
}

fn parse_value(data_type: &str, value: &String) -> Result<Value> {
    debug!("parser_value : {} - {}",data_type,value);
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