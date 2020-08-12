use crate::{Error, Result, Value};

#[function]
pub fn split_csv(line: String) -> Result<Vec<u8>> {
    let mut record = vec![];
    parse_csv(&line, &mut record)?;
    bincode::serialize(&record).or_else(|err| Err(Error::invalid_type(err.to_string())))
}

fn parse_csv(line: &str, values: &mut Vec<String>) -> Result<()> {
    debug!("line - {} \t", line);

    let mut start_index = 0_usize;
    let mut end_index;
    if let Some(col_index) = line.find(",") {
        if col_index > 0 {
            end_index = col_index;
            if let Some(str_start_index) = line.find("'") {
                if str_start_index < col_index {
                    if let Some(str_end_index) = line.split_at(str_start_index + 1).1.find("'") {
                        start_index = str_start_index;
                        end_index = str_start_index + str_end_index + 2
                    }
                }
            }
        } else {
            // 等于 0
            values.push(line.to_string());
            return Ok(());
        }
    } else {
        values.push(line.to_string());
        return Ok(());
    }

    debug!(
        "start_index - {} , end_index - {} \t",
        start_index, end_index
    );

    let rs = (&line[start_index..end_index]).trim().replace("'", "");

    debug!("rs - {}", rs);
    values.push(rs);
    parse_csv(line[(end_index + 1)..].trim(), values)
}

#[test]
fn test() {
    // 1024,'
    // Hello world','
    // Hello world, Good code'
    // ,
    // 20.11,
    // false

    let arg = "'He',1024,'Hello world',10.11,'She',  'Hello world, Good code',20.11,false".to_owned();
    let rs = split_csv(arg).unwrap();
    let values: Vec<&str> = bincode::deserialize(&rs).unwrap();
    assert_eq!(
        [
            "He",
            "1024",
            "Hello world",
            "10.11",
            "She",
            "Hello world, Good code",
            "20.11",
            "false"
        ]
        .to_vec(),
        values
    );
}
