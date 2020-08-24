use crate::{value::Bytes, Columns, Promise, Result, Row, ToData, ToType, Error};
use parking_lot::RwLock;
use ssh::{Session, SftpFile};
use std::{
    alloc::Layout, io::Read, io::Seek, io::SeekFrom, mem::size_of, path::PathBuf, sync::Arc, time::Duration,
};

#[derive(Data)]
pub struct FileBytes {
    file_path: String,
    file_size: i64,
    content: Bytes,
}

#[datasource]
pub fn read_file(
    session: Arc<RwLock<Session>>,
    path: String,
    start_index: i64,
    size: i64,
    timeout: u32,
    promise: &mut Promise<FileBytes>,
) -> Result<()> {
    let mut lock = session
        .try_write_for(Duration::from_secs(timeout as u64))
        .ok_or(Error::lock_faild("lock timeout at 'read_file'"))?;
    let mut sftp = lock.sftp_new()?;
    sftp.init()?;

    let file_path: PathBuf = path.parse()?;

    let mut file = sftp.open(&file_path, libc::O_RDONLY as usize, 0700)?;
    let file_size = file.get_size();

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

    unsafe {
        // 动态分配一个数组大小，用于保存文件内容，该内容大小为 size
        let layout =
            Layout::from_size_align_unchecked((size as usize) * size_of::<u8>(), size_of::<u8>());
        let ptr: *mut u8 = std::alloc::alloc(layout) as *mut u8;
        let buffer = std::slice::from_raw_parts_mut(ptr, size as usize);
        // 避免已分配的内存未被释放
        let rs = read_commit(buffer, &mut file, path, file_size as i64, promise);
        std::alloc::dealloc(ptr as *mut u8, layout);
        // Result 需要在 dealloc 之后处理
        let _ = rs?;
    }
    Ok(())
}

fn read_commit(
    buffer: &mut [u8],
    file: &mut SftpFile,
    path: String,
    file_size: i64,
    promise: &mut Promise<FileBytes>,
) -> Result<()> {
    let _ = file.read(buffer)?;

    promise.commit(FileBytes {
        file_path: path,
        content: buffer.to_vec(),
        file_size,
    })?;

    Ok(())
}

#[test]
fn test() {
    use crate::*;
    let path = "/etc/hosts".to_string();
    let session = super::new_test_sess().unwrap();
    let (req, resp) = crate::new_req(crate::Args::new(), std::time::Duration::from_secs(2));
    {
        let mut promise = req.head::<FileBytes>().unwrap();
        read_file(session, path.clone(), 2, 5,10, &mut promise).unwrap();
        drop(req);
    }

    let resp = resp.wait().unwrap();
    assert_eq!(
        &columns![String: "file_path",Integer: "file_size",Bytes: "content"],
        resp.columns()
    );

    let mut index = 0;
    for row in resp {
        let row = row.unwrap();
        let file_path: String = row.get(0).unwrap();
        let file_size: i64 = row.get(1).unwrap();
        let content: Bytes = row.get(2).unwrap();

        assert_eq!(path, file_path);
        assert!(file_size > 0);
        assert_eq!(5, content.len());
        index += 1;
    }
    assert!(index > 0);
}
