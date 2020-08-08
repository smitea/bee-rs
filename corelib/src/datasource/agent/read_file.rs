use crate::{value::Bytes, Columns, Promise, Result, Row, ToData, ToType};
use std::{fs::File, io::Read, io::Seek, io::SeekFrom, path::PathBuf};

#[derive(Data)]
pub struct FileBytes {
    file_path: String,
    file_size: i64,
    content: Bytes,
}

#[datasource]
pub fn read_file(
    path: String,
    start_index: i64,
    size: i64,
    promise: &mut Promise<FileBytes>,
) -> Result<()> {
    let file_path: PathBuf = path.parse()?;
    let mut file = File::open(&file_path)?;
    let file_size = file.metadata()?.len();
    // 开始位置为负数则从文件末尾开始计算
    let start_seek = if start_index < 0 {
        let start_index = start_index.checked_abs().unwrap_or(0) as u64;
        SeekFrom::Start(file_size - start_index)
    } else {
        SeekFrom::Start(start_index as u64)
    };

    // 设置索引位置
    let start_index = file.seek(start_seek)?;

    let size = if (size as u64) > file_size || size < 0 {
        file_size
    } else {
        size as u64
    };
    debug!(
        "read file [{}] from [{:?}] with size = {}",
        start_index, path, size
    );

    let mut buffer = Vec::with_capacity(size as usize);
    let read_size = file.read(&mut buffer)?;

    promise.commit(FileBytes {
        file_path: path,
        content: buffer,
        file_size: read_size as i64,
    })?;
    Ok(())
}
