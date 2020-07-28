
use crate::{Error, Result, Value};
use csv::ReaderBuilder;

#[function]
pub fn split_space(line: String) -> Result<Vec<u8>> {
    let cols = line
        .split_whitespace()
        .map(|val| val.to_string())
        .collect::<Vec<String>>();
    bincode::serialize(&cols).or_else(|err| Err(Error::invalid_type(err.to_string())))
}

#[function]
pub fn split_csv(line: String) -> Result<Vec<u8>> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(line.as_bytes());

    let mut record = vec![];
    for result in rdr.records() {
        record = result
            .or_else(|err| Err(Error::invalid_type(err.to_string())))?
            .iter()
            .map(|val| val.to_string())
            .collect::<Vec<String>>();
    }

    bincode::serialize(&record).or_else(|err| Err(Error::invalid_type(err.to_string())))
}

#[function]
pub fn get(output: Vec<u8>, index: i32, data_type: String, default: String) -> Result<Value> {
    let array: Vec<String> = bincode::deserialize(&output)
        .or_else(|err| Err(Error::other(0, format!("{}", err))))?;

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