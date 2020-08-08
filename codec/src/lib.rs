use bee_core::{code, DataType, Error, Result, ToType, Value};
use bytes::Buf;
use bytes::BufMut;
use bytes::BytesMut;
pub use connect::{ConnectionReq, ConnectionReqCodec, ConnectionResp, ConnectionRespCodec};
pub use statement::{StatementReq, StatementReqCodec, StatementResp, StatementRespCodec};
use std::io::Read;
use std::{convert::TryFrom, io::Cursor};
use tokio_util::codec::{Decoder, Encoder};

#[macro_use]
extern crate log;

mod connect;
mod statement;

/// 空白内容
pub(crate) const SPACE_BYTE: u8 = 0;
/// 协议头 0xFF 0xFF
pub(crate) const HEAD: &[u8] = &[0xFF, 0xFF];
/// 协议尾 \r\n
pub(crate) const END: &[u8] = &[0x0D, 0x0A];
/// 协议默认长度
pub(crate) const PACKET_LEN: usize = 21;
/// 错误码基址
pub(crate) const INVALID_BASE_CODE: i32 = 0x0C;
pub(crate) const INVALID_DATA_CODE: i32 = code!(INVALID_BASE_CODE, 0x01);

#[repr(u8)]
enum TypeSize {
    NIL = 0x00,
    STRING,
    INTEGER,
    NUMBER,
    BOOLEAN,
    BYTES,
}

impl From<DataType> for TypeSize {
    fn from(d_type: DataType) -> Self {
        match d_type {
            DataType::String => TypeSize::STRING,
            DataType::Integer => TypeSize::INTEGER,
            DataType::Number => TypeSize::NUMBER,
            DataType::Boolean => TypeSize::BOOLEAN,
            DataType::Bytes => TypeSize::BYTES,
            DataType::Nil => TypeSize::NIL,
        }
    }
}

impl Into<DataType> for TypeSize {
    fn into(self) -> DataType {
        match self {
            TypeSize::NIL => DataType::Nil,
            TypeSize::STRING => DataType::String,
            TypeSize::INTEGER => DataType::Integer,
            TypeSize::NUMBER => DataType::Number,
            TypeSize::BOOLEAN => DataType::Boolean,
            TypeSize::BYTES => DataType::Bytes,
        }
    }
}

impl TryFrom<u8> for TypeSize {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self> {
        let d_type = match value {
            0x00 => TypeSize::NIL,
            0x01 => TypeSize::STRING,
            0x02 => TypeSize::INTEGER,
            0x03 => TypeSize::NUMBER,
            0x04 => TypeSize::BOOLEAN,
            0x05 => TypeSize::BYTES,
            _ => {
                return Err(Error::invalid_type(format!(
                    "Invalid type - {} from decode",
                    value
                )))
            }
        };
        Ok(d_type)
    }
}

pub fn write_value<T: Into<Value>>(value: T, data_dist: &mut BytesMut) {
    let value: Value = value.into();

    debug!("write value : {:?}", value);
    match value {
        Value::String(val) => {
            let d_type = TypeSize::STRING as u8;
            let len = val.len() as u32;
            data_dist.put_u8(d_type);
            data_dist.put_u32(len);
            data_dist.extend(val.as_bytes());
        }
        Value::Integer(val) => {
            let d_type = TypeSize::INTEGER as u8;
            data_dist.put_u8(d_type);
            data_dist.put_i64(val);
        }
        Value::Number(val) => {
            let d_type = TypeSize::NUMBER as u8;
            data_dist.put_u8(d_type);
            data_dist.put_f64(val);
        }
        Value::Boolean(val) => {
            let d_type = TypeSize::BOOLEAN as u8;
            data_dist.put_u8(d_type);
            data_dist.put_u8(if val { 0x01 } else { 0x00 });
        }
        Value::Bytes(val) => {
            let d_type = TypeSize::BYTES as u8;
            let len = val.len() as u32;
            data_dist.put_u8(d_type);
            data_dist.put_u32(len);
            data_dist.extend(val);
        }
        Value::Nil => {
            let d_type = TypeSize::NIL as u8;
            data_dist.put_u8(d_type);
        }
    };
}

pub fn read_value<T: TryFrom<Value, Error = Error> + ToType>(
    src: &mut Cursor<&BytesMut>,
) -> Result<T> {
    let value = read_src_value(src)?;
    Ok(T::try_from(value)?)
}

pub fn read_src_value(src: &mut Cursor<&BytesMut>) -> Result<Value> {
    let d_type = TypeSize::try_from(src.get_u8())?;
    let value = match d_type {
        TypeSize::NIL => Value::Nil,
        TypeSize::STRING => {
            let len = src.get_u32();
            let mut bytes: BytesMut = BytesMut::default();
            bytes.resize(len as usize, SPACE_BYTE);
            src.copy_to_slice(&mut bytes);
            Value::from(String::from_utf8(bytes.to_vec())?)
        }
        TypeSize::INTEGER => {
            let value = src.get_i64();
            Value::from(value)
        }
        TypeSize::NUMBER => {
            let value = src.get_f64();
            Value::from(value)
        }
        TypeSize::BOOLEAN => {
            let value = src.get_u8();
            Value::from(if value == 0x01 { true } else { false })
        }
        TypeSize::BYTES => {
            let len = src.get_u32();
            let mut bytes: BytesMut = BytesMut::default();
            bytes.resize(len as usize, SPACE_BYTE);
            src.copy_to_slice(&mut bytes);
            Value::from(bytes.to_vec())
        }
    };
    return Ok(value);
}

pub fn write_error<T: Into<Error>>(value: T, data_dist: &mut BytesMut) {
    let err: Error = value.into();
    // 错误码为 -65535 ~ 65535
    data_dist.put_i32(err.get_code());
    let msg = err.get_msg();
    // 错误内容最大为 255 个字符
    data_dist.put_u8(msg.len() as u8);
    data_dist.extend(msg.as_bytes());
}

pub fn read_error(src: &mut Cursor<&BytesMut>) -> Result<Error> {
    // 获取错误码
    let code = src.get_i32();
    // 获取错误信息
    let msg_len = src.get_u8();
    let mut bytes: BytesMut = BytesMut::default();
    bytes.resize(msg_len as usize, SPACE_BYTE);
    src.copy_to_slice(&mut bytes);
    let msg = String::from_utf8(bytes.to_vec())?;
    Ok(Error::new(code, msg))
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Packet {
    ConnectReq(connect::ConnectionReq),
    ConnectResp(connect::ConnectionResp),
    StatementReq(statement::StatementReq),
    StatementResp(statement::StatementResp),
}

pub struct PacketCodec;

impl Decoder for PacketCodec {
    type Item = Packet;
    type Error = Error;
    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>> {
        let data_size = src.len();
        if data_size >= PACKET_LEN {
            let mut buf: Cursor<&BytesMut> = Cursor::new(&src);
            // Head
            let mut head: BytesMut = BytesMut::new();
            head.resize(HEAD.len(), SPACE_BYTE);
            buf.read(&mut head)?;
            if head.to_vec() != HEAD {
                return Err(Error::other(
                    INVALID_DATA_CODE,
                    format!("invalid head : {:?}", head),
                ));
            }
            let cmd = buf.get_u8();
            let len = buf.get_u64();

            let mut data: BytesMut = BytesMut::new();
            data.resize(len as usize, SPACE_BYTE);
            buf.read(&mut data)?;
            let data = match cmd {
                0x00 => match ConnectionReqCodec.decode(&mut data)? {
                    Some(data) => Packet::ConnectReq(data),
                    None => return Ok(None),
                },
                0x01 => match ConnectionRespCodec.decode(&mut data)? {
                    Some(data) => Packet::ConnectResp(data),
                    None => return Ok(None),
                },
                0x02 => match StatementReqCodec.decode(&mut data)? {
                    Some(data) => Packet::StatementReq(data),
                    None => return Ok(None),
                },
                0x03 => match StatementRespCodec.decode(&mut data)? {
                    Some(data) => Packet::StatementResp(data),
                    None => return Ok(None),
                },
                _ => {
                    return Err(Error::other(
                        INVALID_DATA_CODE,
                        format!("invalid cmd : {}", cmd),
                    ));
                }
            };

            let crc = buf.get_u64();
            if crc != len + (PACKET_LEN as u64) {
                return Err(Error::other(
                    INVALID_DATA_CODE,
                    format!("invalid crc : {:?}", crc),
                ));
            }

            let mut end = BytesMut::new();
            end.resize(2, 0x00);
            buf.read(&mut end)?;
            if end.to_vec() != END {
                return Err(Error::other(
                    INVALID_DATA_CODE,
                    format!("invalid end : {:?}", crc),
                ));
            }
            let position = buf.position() as usize;
            src.advance(position);

            return Ok(Some(data));
        } else {
            return Ok(None);
        }
    }
}

impl Encoder<Packet> for PacketCodec {
    type Error = Error;
    fn encode(&mut self, item: Packet, dst: &mut bytes::BytesMut) -> Result<()> {
        let data = item;

        let mut data_bytes = BytesMut::new();
        let cmd = match data {
            Packet::ConnectReq(req) => {
                ConnectionReqCodec.encode(req, &mut data_bytes)?;
                0x00
            }
            Packet::ConnectResp(resp) => {
                ConnectionRespCodec.encode(resp, &mut data_bytes)?;
                0x01
            }
            Packet::StatementReq(req) => {
                StatementReqCodec.encode(req, &mut data_bytes)?;
                0x02
            }
            Packet::StatementResp(resp) => {
                StatementRespCodec.encode(resp, &mut data_bytes)?;
                0x03
            }
        };

        let len = data_bytes.len() as u64;

        // Head
        dst.put(HEAD);
        // CMD
        dst.put_u8(cmd as u8);
        // Len
        dst.put_u64(len);
        // Data
        dst.extend(data_bytes);
        // CRC
        dst.put_u64((PACKET_LEN as u64) + len);
        // End
        dst.put(END);
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use crate::{
        connect::{ConnectionReq, ConnectionResp},
        statement::{StatementReq, StatementResp},
        Packet, PacketCodec,
    };
    use bee_core::{columns, row, Error, Row};
    use bytes::BytesMut;
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn test_connect_req() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();

        let mut codec = PacketCodec;
        let req = Packet::ConnectReq(ConnectionReq {
            url: "agent://127.0.0.1:6142".to_owned(),
            application: "app1".to_owned(),
        });
        let packet = req;
        let mut dist = BytesMut::new();
        codec.encode(packet.clone(), &mut dist).unwrap();

        info!("{:x}", dist);
        assert_eq!(
            b"\xff\xff\0\0\0\0\0\0\0\0$\x01\0\0\0\x16agent://127.0.0.1:6142\x01\0\0\0\x04app1\0\0\0\0\0\0\09\r\n".to_vec(),
            dist
        );
        let rs = codec.decode(&mut dist).unwrap().unwrap();
        assert_eq!(rs, packet);
    }

    #[test]
    fn test_connect_resp() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();

        let mut codec = PacketCodec;
        let req = Packet::ConnectResp(ConnectionResp::Ok);
        let packet = req;
        let mut dist = BytesMut::new();
        codec.encode(packet.clone(), &mut dist).unwrap();
        info!("{:x}", dist);
        assert_eq!(
            b"\xff\xff\x01\0\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\0\x16\r\n".to_vec(),
            dist
        );
        let rs = codec.decode(&mut dist).unwrap().unwrap();
        assert_eq!(rs, packet);

        let resp = Packet::ConnectResp(ConnectionResp::Error(Error::new(0x01, "Failed!")));
        let packet = resp;
        let mut dist = BytesMut::new();
        codec.encode(packet.clone(), &mut dist).unwrap();
        info!("{:x}", dist);
        assert_eq!(
            b"\xff\xff\x01\0\0\0\0\0\0\0\r\x01\0\0\0\x01\x07Failed!\0\0\0\0\0\0\0\"\r\n".to_vec(),
            dist
        );
        let rs = codec.decode(&mut dist).unwrap().unwrap();
        assert_eq!(rs, packet);
    }

    #[test]
    fn test_statement_req() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();

        let mut codec = PacketCodec;
        let req = Packet::StatementReq(StatementReq {
            id: 01,
            script: "SELECT *FROM m_test()".to_owned(),
            timeout: 10,
        });
        let packet = req;
        let mut dist = BytesMut::new();
        codec.encode(packet.clone(), &mut dist).unwrap();
        assert_eq!(
            b"\xff\xff\x02\0\0\0\0\0\0\0,\x02\0\0\0\0\0\0\0\x01\x01\0\0\0\x15SELECT *FROM m_test()\x02\0\0\0\0\0\0\0\n\0\0\0\0\0\0\0A\r\n".to_vec(),
            dist
        );
        let rs = codec.decode(&mut dist).unwrap().unwrap();
        assert_eq!(rs, packet);
    }

    #[test]
    fn test_statement_resp() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();

        let mut codec = PacketCodec;

        let columns = columns![
            String  : "Name",
            Number  : "Age",
            Integer : "Count",
            Boolean : "IsNice",
            Bytes   : "Image",
            Nil     : "Phone"
        ];
        let resp = StatementResp::Columns(columns);
        let packet = Packet::StatementResp(resp);
        let mut dist = BytesMut::new();
        codec.encode(packet.clone(), &mut dist).unwrap();
        assert_eq!(
            b"\xff\xff\x03\0\0\0\0\0\0\0*\0\x06\x04Name\x01\x03Age\x03\x05Count\x02\x06IsNice\x04\x05Image\x05\x05Phone\0\0\0\0\0\0\0\0?\r\n"
                .to_vec(),
            dist
        );
        let rs = codec.decode(&mut dist).unwrap().unwrap();
        assert_eq!(rs, packet);

        let row: Row = row!(10, 20.0, "Name", false, vec![0x01, 0x02]);
        let req = StatementResp::Row(row);
        let packet = Packet::StatementResp(req);
        let mut dist = BytesMut::new();
        codec.encode(packet.clone(), &mut dist).unwrap();
        assert_eq!(b"\xff\xff\x03\0\0\0\0\0\0\0&\x01\x05\x02\0\0\0\0\0\0\0\n\x03@4\0\0\0\0\0\0\x01\0\0\0\x04Name\x04\0\x05\0\0\0\x02\x01\x02\0\0\0\0\0\0\0;\r\n".to_vec(), dist);
        let rs = codec.decode(&mut dist).unwrap().unwrap();
        assert_eq!(rs, packet);

        let req = StatementResp::Error(Error::new(0x12, "failed to!"));
        let packet = Packet::StatementResp(req);
        let mut dist = BytesMut::new();
        codec.encode(packet.clone(), &mut dist).unwrap();
        assert_eq!(
            b"\xff\xff\x03\0\0\0\0\0\0\0\x10\x03\0\0\0\x12\nfailed to!\0\0\0\0\0\0\0%\r\n".to_vec(),
            dist
        );
        let rs = codec.decode(&mut dist).unwrap().unwrap();
        assert_eq!(rs, packet);

        let req = StatementResp::Abort;
        let mut dist = BytesMut::new();
        let packet = Packet::StatementResp(req);
        codec.encode(packet.clone(), &mut dist).unwrap();
        assert_eq!(
            b"\xff\xff\x03\0\0\0\0\0\0\0\x01\x02\0\0\0\0\0\0\0\x16\r\n".to_vec(),
            dist
        );
        let rs = codec.decode(&mut dist).unwrap().unwrap();
        assert_eq!(rs, packet);
    }
}
