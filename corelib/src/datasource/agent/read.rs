use crate::{Promise, Result,ToData,ToType, datasource::FileLine};
use std::{fs::File, io::BufRead, io::BufReader, io::Seek, io::SeekFrom, path::PathBuf};

#[datasource]
pub fn read_agent_file(
    path: String,
    start_index: i64,
    end_index: i64,
    promise: &mut Promise<FileLine>,
) -> Result<()> {
    let path: PathBuf = path.parse()?;
    let mut file = File::open(path)?;
    let file_size = file.metadata()?.len();
    // 开始位置为负数则从文件末尾开始计算
    let start_seek = if start_index < 0 {
        let start_index = start_index.checked_abs().unwrap_or(0) as u64;
        SeekFrom::Start(file_size - start_index)
    } else {
        SeekFrom::Start(start_index as u64)
    };

    // 结束位置为
    let end_seek = SeekFrom::End(end_index);

    // 设置索引位置
    file.seek(start_seek)?;
    file.seek(end_seek)?;

    let reader = BufReader::new(file);
    let lines = reader.lines();

    let mut line_num = 0;
    for line in lines {
        promise.commit(FileLine {
            line: line?,
            line_num,
        })?;
        line_num += 1;
    }

    Ok(())
}
