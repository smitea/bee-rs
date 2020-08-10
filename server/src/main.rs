use bee_codec::*;
use bee_core::new_connection;
use bee_core::Connection;
use colored::*;
use log::{error, info};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpListener;
use tokio::stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};

use futures;
use futures::SinkExt;
use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use std::{io::ErrorKind, path::Path};
use structopt::StructOpt;

const BEE: &str = "bee";
const CONNECT: &str = "connection";
const REQUEST: &str = "statements";

fn setup_logger(level: &str) -> Result<(), fern::InitError> {
    let current_dir = std::env::current_exe()?;
    let current_dir: &Path = current_dir.parent().unwrap();

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{0}[{1: <5}][{2: <5}] {3}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::from_str(level).unwrap())
        .chain(std::io::stdout())
        .chain(fern::log_file(current_dir.join("bee.log"))?)
        .apply()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli: CLI = CLI::from_args();

    match cli {
        CLI::Run(config) => start(config).await?,
    }

    Ok(())
}

#[derive(Debug, StructOpt)]
enum CLI {
    #[structopt(name = "run")]
    Run(Config),
}

#[derive(Debug, StructOpt)]
struct Config {
    #[structopt(long = "ip", default_value = "0.0.0.0")]
    ip: String,
    #[structopt(long = "port", default_value = "6142")]
    port: u16,
    #[structopt(long = "log_level", default_value = "Info")]
    log_level: String,
    #[structopt(long = "work_size", default_value = "4")]
    work_size: usize,
}

async fn start(config: Config) -> Result<(), Box<dyn Error>> {
    let addr = format!("{}:{}", config.ip, config.port);
    setup_logger(&config.log_level)?;
    print_headers(&config)?;

    let mut listener = TcpListener::bind(&addr).await?;

    loop {
        let (stream, addr) = listener.accept().await?;
        tokio::spawn(async move {
            info!(target: CONNECT, "{} - connected", addr);
            let (reader, writer) = stream.into_split();
            let reader_framed = FramedRead::new(reader, PacketCodec);
            let writer_framed = FramedWrite::new(writer, PacketCodec);
            if let Err(e) = process(reader_framed, writer_framed, addr).await {
                println!("an error occurred; error = {:?}", e);
            }
        });
    }
}

fn print_headers(config: &Config) -> Result<(), Box<dyn Error>> {
    let version: String = env!("CARGO_PKG_VERSION").to_string();
    let banner: String = r#"
  __     
 / _)_ _ 
/(_)(-(- "#
        .to_owned();
    println!("{}    {}", banner.color("yellow"), version.color("green"));
    println!();
    info!(target: BEE, "log level            {}", config.log_level);
    info!(target: BEE, "work size            {}", config.work_size);
    info!(target: BEE, "--------------------------------");
    info!(
        target: BEE,
        "{}",
        format!("listener on          {}:{} ...", config.ip, config.port).color("green")
    );
    info!(target: BEE, "");
    Ok(())
}

async fn process<'a>(
    mut reader_framed: FramedRead<OwnedReadHalf, PacketCodec>,
    mut writer_framed: FramedWrite<OwnedWriteHalf, PacketCodec>,
    addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let (connection, app) = if let Some(Ok(Packet::ConnectReq(req))) = reader_framed.next().await {
        info!(
            target: CONNECT,
            "[{}] - connecting to {} ...", req.application, req.url
        );
        (new_connection(&req.url)?, req.application)
    } else {
        return Err(Box::new(std::io::Error::new(
            ErrorKind::Other,
            format!("{} - doesn't connection", addr),
        )));
    };

    info!(target: CONNECT, "[{}] - connection", app);
    while let Some(Ok(Packet::StatementReq(req))) = reader_framed.next().await {
        match new_statement(&connection, &app, &req, &mut writer_framed).await {
            Ok(_) => {
                error!(target: REQUEST, "[{}-{}] is ok", app, req.id);
                // 采集结束
                writer_framed
                    .send(Packet::StatementResp(StatementResp::new(
                        StatementStateResp::Abort,
                        req.id,
                    )))
                    .await?;
            }
            Err(err) => {
                error!(target: REQUEST, "[{}-{}] is failed : {}", app, req.id, err);
                // 采集错误
                writer_framed
                    .send(Packet::StatementResp(StatementResp::new(
                        StatementStateResp::Error(err),
                        req.id,
                    )))
                    .await?;
            }
        }
    }
    info!(target: CONNECT, "[{}] - disconnection", app);
    Ok(())
}

async fn new_statement<'a>(
    connection: &Box<dyn Connection>,
    app_name: &str,
    req: &StatementReq,
    writer_framed: &mut FramedWrite<OwnedWriteHalf, PacketCodec>,
) -> Result<(), bee_core::Error> {
    info!(target: REQUEST, "[{}-{}] new statement", app_name, req.id);
    let statement =
        connection.new_statement(&req.script, Duration::from_secs(req.timeout as u64))?;
    let response = statement.wait()?;
    let columns = response.columns();
    info!(
        target: REQUEST,
        "[{}-{}] resp columns - {:?}", app_name, req.id, columns
    );
    // 应答列的结构定义
    writer_framed
        .send(Packet::StatementResp(StatementResp::new(
            StatementStateResp::Columns(columns.clone()),
            req.id,
        )))
        .await?;

    for rs in response {
        let row = rs?;
        info!(
            target: REQUEST,
            "[{}-{}] resp row - {:?}", app_name, req.id, row
        );
        writer_framed
            .send(Packet::StatementResp(StatementResp::new(
                StatementStateResp::Row(row),
                req.id,
            )))
            .await?;
    }
    Ok(())
}
