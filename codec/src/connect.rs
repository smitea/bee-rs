use crate::{read_error, read_value, write_error, write_value};
use bee_core::{Error, Result};
use bytes::Buf;
use bytes::BufMut;
use bytes::BytesMut;
use std::io::Cursor;
use tokio_util::codec::Decoder;
use tokio_util::codec::Encoder;

#[derive(Debug, Clone,Eq, PartialEq)]
pub struct ConnectionReq {
    pub url: String,
    pub application: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ConnectionResp {
    Ok,
    Error(Error),
}

pub struct ConnectionReqCodec;

impl Decoder for ConnectionReqCodec {
    type Item = ConnectionReq;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        let mut buf: Cursor<&BytesMut> = Cursor::new(&src);
        let url: String = read_value(&mut buf)?;
        let application: String = read_value(&mut buf)?;
        Ok(Some(ConnectionReq { url, application }))
    }
}

impl Encoder<ConnectionReq> for ConnectionReqCodec {
    type Error = Error;
    fn encode(&mut self, item: ConnectionReq, dst: &mut BytesMut) -> Result<()> {
        write_value(item.url, dst);
        write_value(item.application, dst);
        Ok(())
    }
}

pub struct ConnectionRespCodec;

impl Decoder for ConnectionRespCodec {
    type Item = ConnectionResp;
    type Error = Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        let mut buf: Cursor<&BytesMut> = Cursor::new(&src);
        let resp = buf.get_u8();

        let resp = match resp {
            0x00 => ConnectionResp::Ok,
            0x01 => ConnectionResp::Error(read_error(&mut buf)?),
            _ => {
                return Err(Error::invalid_type(format!(
                    "Invalid type - {} from decode",
                    resp
                )))
            }
        };

        Ok(Some(resp))
    }
}

impl Encoder<ConnectionResp> for ConnectionRespCodec {
    type Error = Error;
    fn encode(&mut self, item: ConnectionResp, dist: &mut BytesMut) -> Result<()> {
        match item {
            ConnectionResp::Ok => dist.put_u8(0x00),
            ConnectionResp::Error(err) => {
                dist.put_u8(0x01);
                write_error(err, dist);
            }
        }
        Ok(())
    }
}

#[test]
fn test_connection_req() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .try_init();

    let mut codec = ConnectionReqCodec;
    let req = ConnectionReq {
        url: "agent://127.0.0.1:6142".to_owned(),
        application: "app1".to_owned(),
    };

    let mut dist = BytesMut::new();
    codec.encode(req.clone(), &mut dist).unwrap();

    assert_eq!(
        b"\x01\0\0\0\x16agent://127.0.0.1:6142\x01\0\0\0\x04app1".to_vec(),
        dist
    );

    println!("{:x}",dist);
    let rs = codec.decode(&mut dist).unwrap().unwrap();
    assert_eq!(&rs.application, &req.application);
    assert_eq!(&rs.url, &req.url);
}

#[test]
fn test_connection_resp() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .try_init();

    let mut codec = ConnectionRespCodec;
    let req = ConnectionResp::Ok;
    let mut dist = BytesMut::new();
    codec.encode(req.clone(), &mut dist).unwrap();
    assert_eq!(b"\x00".to_vec(), dist);
    let rs = codec.decode(&mut dist).unwrap().unwrap();
    assert_eq!(rs, req);

    let mut codec = ConnectionRespCodec;
    let req = ConnectionResp::Error(Error::new(0x01, "Failed!"));
    let mut dist = BytesMut::new();
    codec.encode(req.clone(), &mut dist).unwrap();
    assert_eq!(b"\x01\x00\x00\x00\x01\x07Failed!".to_vec(), dist);
    let rs = codec.decode(&mut dist).unwrap().unwrap();
    assert_eq!(rs, req);
}
