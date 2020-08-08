use crate::{datasource::BashRow, Promise, Result, ToData, ToType};
use ssh::Session;
use std::{
    io::BufRead, io::BufReader, io::Seek, io::SeekFrom, path::PathBuf, sync::Arc, sync::Mutex,
};

#[datasource]
pub fn read_remote_file(
    session: Arc<Mutex<Session>>,
    path: String,
    start_index: i64,
    end_index: i64,
    promise: &mut Promise<BashRow>,
) -> Result<()> {
    let mut lock = session.lock()?;
    let mut sftp = lock.sftp_new()?;

    let path: PathBuf = path.parse()?;
    let mut file = sftp.open(path, libc::O_RDONLY as usize, 0700)?;

    let file_size = file.stream_len()?;
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
    let _ = file.seek(start_seek)?;
    let _ = file.seek(end_seek)?;

    let reader = BufReader::new(file);
    let lines = reader.lines();

    let mut line_num = 0;
    for line in lines {
        promise.commit(BashRow {
            line: line?,
            line_num,
        })?;
        line_num += 1;
    }

    Ok(())
}
