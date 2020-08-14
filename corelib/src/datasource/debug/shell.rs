use crate::{BashRow, Promise, Result, ToData, ToType};

#[datasource]
fn shell(output: String, _timeout: u32, promise: &mut Promise<BashRow>) -> Result<()> {
    let lines = output.lines();

    let mut index = 0;
    for line in lines {
        promise.commit(BashRow {
            line: line.to_string(),
            line_num: index,
        })?;
        index += 1;
    }
    Ok(())
}
