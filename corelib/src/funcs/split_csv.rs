use crate::{Error, Result, Value};
use csv::ReaderBuilder;

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
