use crate::{
    read_error, read_src_value, read_value, write_error, write_value, TypeSize, SPACE_BYTE,
};
use bee_core::Error;
use bee_core::{Columns, DataType, Result, Row, State, Value};
use bytes::Buf;
use bytes::BufMut;
use bytes::BytesMut;
use std::{convert::TryFrom, io::Cursor};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StatementReq {
    pub id: u32,
    pub script: String,
    pub timeout: u32,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StatementResp {
    Columns(Columns),
    Row(Row),
    Abort,
    Error(Error),
}

#[repr(u8)]
pub enum StatementType {
    Columns = 0x00,
    Row,
    Abort,
    Error,
}

impl From<State> for StatementResp {
    fn from(state: State) -> Self {
        match state {
            State::Ready(columns) => StatementResp::Columns(columns),
            State::Process(row) => StatementResp::Row(row),
            State::Err(err) => StatementResp::Error(err),
            State::Ok => StatementResp::Abort,
        }
    }
}

impl TryFrom<u8> for StatementType {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self> {
        let s_type = match value {
            0x00 => StatementType::Columns,
            0x01 => StatementType::Row,
            0x02 => StatementType::Abort,
            0x03 => StatementType::Error,
            _ => {
                return Err(Error::invalid_type(format!(
                    "Invalid type - {} from decode",
                    value
                )))
            }
        };

        Ok(s_type)
    }
}

pub struct StatementReqCodec;

impl Decoder for StatementReqCodec {
    type Item = StatementReq;
    type Error = Error;
    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>> {
        let mut buf: Cursor<&BytesMut> = Cursor::new(&src);
        let id: u32 = read_value(&mut buf)?;
        let script: String = read_value(&mut buf)?;
        let timeout: u32 = read_value(&mut buf)?;

        Ok(Some(StatementReq {
            id,
            script,
            timeout,
        }))
    }
}

impl Encoder<StatementReq> for StatementReqCodec {
    type Error = Error;
    fn encode(&mut self, item: StatementReq, dst: &mut bytes::BytesMut) -> Result<()> {
        write_value(item.id, dst);
        write_value(item.script, dst);
        write_value(item.timeout, dst);

        Ok(())
    }
}

pub struct StatementRespCodec;

impl Decoder for StatementRespCodec {
    type Item = StatementResp;
    type Error = Error;
    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>> {
        let mut buf: Cursor<&BytesMut> = Cursor::new(&src);
        let statemtn_type = StatementType::try_from(buf.get_u8())?;

        let statement = match statemtn_type {
            StatementType::Columns => {
                let col_size = buf.get_u8();
                let mut values = Columns::new();
                for _ in 0..col_size {
                    // 获取列名
                    let col_name_len = buf.get_u8();
                    let mut bytes: BytesMut = BytesMut::default();
                    bytes.resize(col_name_len as usize, SPACE_BYTE);
                    buf.copy_to_slice(&mut bytes);
                    let name = String::from_utf8(bytes.to_vec())?;

                    // 获取列类型
                    let d_type = buf.get_u8();
                    let d_type = TypeSize::try_from(d_type)?;
                    let d_type: DataType = d_type.into();
                    values.push(name, d_type);
                }

                StatementResp::Columns(values)
            }
            StatementType::Row => {
                let len = buf.get_u8();
                let mut row = Row::new();
                for _ in 0..len {
                    let value: Value = read_src_value(&mut buf)?;
                    row.push(value);
                }
                StatementResp::Row(row)
            }
            StatementType::Abort => StatementResp::Abort,
            StatementType::Error => StatementResp::Error(read_error(&mut buf)?),
        };

        Ok(Some(statement))
    }
}

impl Encoder<StatementResp> for StatementRespCodec {
    type Error = Error;
    fn encode(&mut self, item: StatementResp, dst: &mut bytes::BytesMut) -> Result<()> {
        match item {
            StatementResp::Columns(columns) => {
                let values = columns.to_vec();
                dst.put_u8(StatementType::Columns as u8);
                // 限制最大为 255 列
                dst.put_u8(values.len() as u8);
                for (name, d_type) in values {
                    // 列名限制最大长度为 255 个字符
                    dst.put_u8(name.len() as u8);
                    dst.extend(name.as_bytes());
                    dst.put_u8(TypeSize::from(d_type) as u8);
                }
            }
            StatementResp::Row(row) => {
                let values = row.to_vec();
                dst.put_u8(StatementType::Row as u8);
                // 限制最大为 255 列
                dst.put_u8(values.len() as u8);
                for value in values {
                    write_value(value, dst);
                }
            }
            StatementResp::Abort => {
                dst.put_u8(StatementType::Abort as u8);
            }
            StatementResp::Error(err) => {
                dst.put_u8(StatementType::Error as u8);
                write_error(err, dst);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::{StatementReq, StatementReqCodec, StatementResp, StatementRespCodec};
    use bee_core::{columns, Error, Row, row};
    use bytes::BytesMut;
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn test_req() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();

        let mut codec = StatementReqCodec;
        let req = StatementReq {
            id: 01,
            script: "SELECT *FROM m_test()".to_owned(),
            timeout: 10,
        };

        let mut dist = BytesMut::new();
        codec.encode(req.clone(), &mut dist).unwrap();
        info!("{:x}", dist);
        assert_eq!(
            b"\x02\0\0\0\0\0\0\0\x01\x01\0\0\0\x15SELECT *FROM m_test()\x02\0\0\0\0\0\0\0\x0A"
                .to_vec(),
            dist
        );

        let rs = codec.decode(&mut dist).unwrap().unwrap();
        assert_eq!(req.id, rs.id);
        assert_eq!(req.script, rs.script);
        assert_eq!(req.timeout, rs.timeout);
    }

    #[test]
    fn test_reps() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();

        let mut codec = StatementRespCodec;

        let columns = columns![
            String  : "Name",
            Number  : "Age",
            Integer : "Count",
            Boolean : "IsNice",
            Bytes   : "Image",
            Nil     : "Phone"
        ];
        let req = StatementResp::Columns(columns);
        let mut dist = BytesMut::new();
        codec.encode(req.clone(), &mut dist).unwrap();
        assert_eq!(
            b"\0\x06\x04Name\x01\x03Age\x03\x05Count\x02\x06IsNice\x04\x05Image\x05\x05Phone\0"
                .to_vec(),
            dist
        );
        let rs = codec.decode(&mut dist).unwrap().unwrap();
        assert_eq!(rs, req);

        let row: Row = row!(10, 20.0, "Name", false, vec![0x01, 0x02]);
        let req = StatementResp::Row(row);
        let mut dist = BytesMut::new();
        codec.encode(req.clone(), &mut dist).unwrap();
        info!("{:x}", dist);
        assert_eq!(b"\x01\x05\x02\0\0\0\0\0\0\0\n\x03@4\0\0\0\0\0\0\x01\0\0\0\x04Name\x04\0\x05\0\0\0\x02\x01\x02".to_vec(), dist);
        let rs = codec.decode(&mut dist).unwrap().unwrap();
        assert_eq!(rs, req);

        let req = StatementResp::Error(Error::new(0x12, "failed to!"));
        let mut dist = BytesMut::new();
        codec.encode(req.clone(), &mut dist).unwrap();
        assert_eq!(b"\x03\0\0\0\x12\x0Afailed to!".to_vec(), dist);
        let rs = codec.decode(&mut dist).unwrap().unwrap();
        assert_eq!(rs, req);

        let req = StatementResp::Abort;
        let mut dist = BytesMut::new();
        codec.encode(req.clone(), &mut dist).unwrap();
        assert_eq!(b"\x02".to_vec(), dist);
        let rs = codec.decode(&mut dist).unwrap().unwrap();
        assert_eq!(rs, req);
    }
}
