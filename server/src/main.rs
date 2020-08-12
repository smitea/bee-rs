use bee_codec::*;
use bee_core::new_connection;
use bee_core::{columns, new_req, row, Args, Connection, Promise, Statement, ToData};
use colored::*;
use log::{debug, error, info};
use std::result::Result;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpListener;
use tokio::stream::StreamExt;
use tokio::sync::Mutex;

use futures;
use futures::SinkExt;
use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use std::{collections::HashMap, io::ErrorKind, path::Path, sync::Arc};
use structopt::StructOpt;

const BEE: &str = "bee";
const CONNECT: &str = "connection";
const REQUEST: &str = "statements";
const QUERY_STATE_SQL: &str = "show network_states";

type State = Arc<Mutex<HashMap<SocketAddr, ClientInfo>>>;

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
}

#[derive(Clone)]
struct ClientInfo {
    addr: SocketAddr,
    connect: ConnectionReq,
    state: ClientState,
}

#[derive(Clone)]
enum ClientState {
    Idle(StatementReq, f64),
    Process(StatementReq),
    New,
}

impl ToData for ClientInfo {
    fn columns() -> bee_core::Columns {
        columns![String: "addr",String: "application", String: "url", Integer: "sid", String: "script",Number: "used(s)", String: "status"]
    }
    fn to_row(self) -> bee_core::Row {
        let mut row = row![
            self.addr.to_string(),
            self.connect.application,
            self.connect.url
        ];

        match self.state {
            ClientState::Idle(req, used) => {
                row.push(req.id);
                row.push(req.script);
                row.push(used);
                row.push("idle");
            }
            ClientState::Process(req) => {
                row.push(req.id);
                row.push(req.script);
                row.push(());
                row.push("process");
            }
            ClientState::New => {
                row.push(());
                row.push("".to_string());
                row.push(());
                row.push("new");
            }
        }
        row
    }
}

async fn start(config: Config) -> Result<(), Box<dyn Error>> {
    setup_logger(&config.log_level)?;
    print_headers(&config)?;

    let addr = format!("{}:{}", config.ip, config.port);
    let mut listener = TcpListener::bind(&addr).await?;
    let clients: State = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let (stream, addr) = listener.accept().await?;
        stream.set_nodelay(true)?;
        stream.set_keepalive(Some(Duration::from_secs(10)))?;
        stream.set_recv_buffer_size(1024 * 10)?;
        stream.set_send_buffer_size(1024 * 10)?;

        let state = clients.clone();
        tokio::spawn(async move {
            info!(target: CONNECT, "{} - connected", addr);
            let (reader, writer) = stream.into_split();
            let reader_framed = FramedRead::new(reader, PacketCodec);
            let writer_framed = FramedWrite::new(writer, PacketCodec);
            if let Err(e) = process(state.clone(), reader_framed, writer_framed, addr).await {
                info!("an error occurred; error = {:?}", e);
            }
            {
                // 移除连接信息
                let mut state = state.lock().await;
                state.remove(&addr);
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
    println!("");
    info!(target: BEE, "log level            {}", config.log_level);
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
    state: State,
    mut reader_framed: FramedRead<OwnedReadHalf, PacketCodec>,
    mut writer_framed: FramedWrite<OwnedWriteHalf, PacketCodec>,
    addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    // 等待 请求连接 数据包
    let (connection, req) = if let Some(Ok(Packet::ConnectReq(req))) = reader_framed.next().await {
        info!(
            target: CONNECT,
            "[{}] - connecting to {} ...", req.application, req.url
        );
        match new_connection(&req.url) {
            Ok(connection) => {
                writer_framed
                    .send(Packet::ConnectResp(ConnectionResp::Ok))
                    .await?;
                (connection, req)
            }
            Err(err) => {
                writer_framed
                    .send(Packet::ConnectResp(ConnectionResp::Error(err.clone())))
                    .await?;
                return Err(Box::new(err));
            }
        }
    } else {
        return Err(Box::new(std::io::Error::new(
            ErrorKind::Other,
            format!("{} - doesn't connection", addr),
        )));
    };

    // 记录连接信息
    {
        let mut state = state.lock().await;
        state.insert(
            addr,
            ClientInfo {
                addr,
                connect: req.clone(),
                state: ClientState::New,
            },
        );
    }

    let app = req.application;
    info!(target: CONNECT, "[{}] - connected.", app);
    while let Some(Ok(Packet::StatementReq(req))) = reader_framed.next().await {
        {
            if req.script != QUERY_STATE_SQL {
                // 更新状态信息
                let mut state = state.lock().await;
                state.entry(addr).and_modify(|c| {
                    c.state = ClientState::Process(req.clone());
                });
            }
        }
        let now = SystemTime::now();
        match new_statement(&connection, state.clone(), &app, &req, &mut writer_framed).await {
            Ok(_) => {
                info!(target: REQUEST, "[{}-{}] is ok", app, req.id);
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
        };
        let elapsed = now.elapsed()?;
        let used = elapsed.as_secs_f64();
        {
            if req.script != QUERY_STATE_SQL {
                // 更新状态信息
                let mut state = state.lock().await;
                state.entry(addr).and_modify(|c| {
                    c.state = ClientState::Idle(req.clone(), used);
                });
            }
        }
    }
    info!(target: CONNECT, "[{}] - disconnected.", app);
    Ok(())
}

async fn new_statement<'a>(
    connection: &Box<dyn Connection>,
    state: State,
    app_name: &str,
    req: &StatementReq,
    writer_framed: &mut FramedWrite<OwnedWriteHalf, PacketCodec>,
) -> Result<(), bee_core::Error> {
    info!(
        target: REQUEST,
        "[{}-{}] process {} in {} s.", app_name, req.id, req.script, req.timeout
    );

    let statement = if req.script == QUERY_STATE_SQL {
        // 返回当前所有连接的执行状态
        network_states_resp(state, req).await?
    } else {
        connection.new_statement(&req.script, Duration::from_secs(req.timeout as u64))?
    };
    let response = statement.wait()?;
    let columns = response.columns();

    let mut row_count = 0;
    debug!(
        target: REQUEST,
        "[{}-{}] responsed columns - {:?}", app_name, req.id, columns
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
        debug!(
            target: REQUEST,
            "[{}-{}] responsed row - {:?}", app_name, req.id, row
        );
        row_count += 1;
        writer_framed
            .send(Packet::StatementResp(StatementResp::new(
                StatementStateResp::Row(row),
                req.id,
            )))
            .await?;
    }
    info!(
        target: REQUEST,
        "[{}-{}] responsed {} rows.", app_name, req.id, row_count
    );
    Ok(())
}

async fn network_states_resp(
    state: State,
    req: &StatementReq,
) -> Result<Statement, bee_core::Error> {
    let (request, response) = new_req(Args::new(), Duration::from_secs(req.timeout as u64));
    let mut commit: Promise<ClientInfo> = request.head()?;
    {
        let lock = state.lock().await;
        for (_, value) in lock.iter() {
            commit.commit(value.clone())?;
        }
    }
    Ok(response)
}
