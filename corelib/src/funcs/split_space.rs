use crate::{Error, Result, Value};

#[function]
pub fn split_space(line: String) -> Result<Vec<u8>> {
    let cols = line
        .split_whitespace()
        .map(|val| val.to_string())
        .collect::<Vec<String>>();
    bincode::serialize(&cols).or_else(|err| Err(Error::invalid_type(err.to_string())))
}
