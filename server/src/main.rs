use bee_codec::*;
use bee_core::new_connection;
use bee_core::{columns, new_req, row, Args, Connection, Promise, Statement, ToData};
use colored::*;
use log::{debug, error, info};
use std::result::Result;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpListener;
use tokio::stream::StreamExt;
use tokio::{runtime::Runtime, sync::Mutex};

use futures;
use futures::SinkExt;
use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use std::{collections::HashMap, io::ErrorKind, path::Path, sync::Arc};
use structopt::StructOpt;

#[cfg(windows)]
#[macro_use]
extern crate windows_service;
#[cfg(windows)]
define_windows_service!(ffi_service_main, start_service);

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

const HIVE: &str = PKG_NAME;
const CONNECT: &str = "connection";
const REQUEST: &str = "statements";
const QUERY_NETWORK_STATES: &str = "show network_states";

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
        .chain(fern::log_file(current_dir.join(format!("{}.log", HIVE)))?)
        .apply()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    start()?;
    Ok(())
}

#[cfg(unix)]
#[derive(Debug, StructOpt)]
enum CLI {
    #[structopt(name = "run")]
    Run(Config),
    #[structopt(name = "start")]
    Start(Config),
    #[structopt(name = "stop")]
    Stop,
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

#[cfg(unix)]
fn start() -> Result<(), Box<dyn Error>> {
    let cli: CLI = CLI::from_args();
    match cli {
        CLI::Run(config) => run(config),
        CLI::Start(config) => start_service(config),
        CLI::Stop => stop(),
    }
}

#[cfg(unix)]
fn start_service(config: Config) -> Result<(), Box<dyn Error>> {
    let pid_path = get_pid_path();
    if let Ok(pid) = std::fs::read_to_string(&pid_path) {
        eprintln!("The {} is started, and PID: {}", HIVE, pid.red());
        return Ok(());
    }

    let daemonize = daemonize::Daemonize::new()
        .working_directory(
            replace_env("${CWD}").expect("failed to replace current_dir for work_directory"),
        )
        .exit_action(move || {
            // 等待 1 sec 后读取 PID
            std::thread::sleep(Duration::from_secs(1));
            let pid_path = get_pid_path();

            match std::fs::read_to_string(&pid_path) {
                Ok(pid) => println!("Success to start of {}, and PID: {}", HIVE, pid.green()),
                Err(err) => {
                    eprintln!("Failed to open PID file [{:?}] - {}", &pid_path, err);
                }
            }
        })
        .privileged_action(move || {
            let pid_path = get_pid_path();
            if let Err(err) = std::fs::write(&pid_path, format!("{}", std::process::id())) {
                error!("Failed to write PID file [{:?}] - {}", &pid_path, err);
            }
            if let Err(err) = run(config) {
                error!("{}", err);
            }
        });
    match daemonize.start() {
        Ok(_) => println!("Success to start of {}", HIVE),
        Err(e) => eprintln!("Error, {}", e),
    }
    Ok(())
}

#[cfg(unix)]
fn get_pid_path() -> std::path::PathBuf {
    replace_env("${CWD}/.pid").expect("Failed to replace current_dir for [.pid]")
}

#[cfg(windows)]
fn start() -> Result<(), Box<dyn Error>> {
    windows_service::service_dispatcher::start(HIVE, ffi_service_main)?;
    Ok(())
}

#[cfg(windows)]
fn start_service(arguments: Vec<std::ffi::OsString>) {
    if let Err(e) = run_service(arguments) {
        error!("{}", e);
    }
}

#[cfg(windows)]
pub fn run_service(arguments: Vec<std::ffi::OsString>) -> Result<(), Box<dyn Error>> {
    use windows_service::{
        service::{
            ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
    };

    let config = {
        let mut log_level = "Info".to_owned();
        let mut ip = "0.0.0.0".to_owned();
        let mut port = 6142_u16;
        for arg in arguments {
            let arg = arg.to_str().unwrap_or("");
            if arg.contains("--log_level=") {
                log_level = arg.replace("--log_level=", "");
            } else if arg.contains("--ip=") {
                ip = arg.replace("--ip=", "");
            } else if arg.contains("--port=") {
                port = arg.replace("--port=", "").trim().parse()?;
            }
        }

        Config {
            log_level,
            ip,
            port,
        }
    };
    let (tx, rx) = std::sync::mpsc::channel();

    let sin_tx = tx.clone();
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Interrogate | ServiceControl::Stop => {
                let _ = tx.send((ServiceState::Stopped, 0));
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(HIVE, event_handler)?;
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    std::thread::spawn(move || {
        if let Err(err) = run(config) {
            error!("{}", err);
            let _ = sin_tx.send((ServiceState::Stopped, 1));
        } else {
            let _ = sin_tx.send((ServiceState::Stopped, 0));
        };
    });

    if let Ok((state, code)) = rx.recv() {
        status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: state,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(code),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;
    };

    Ok(())
}

#[cfg(unix)]
fn stop() -> Result<(), Box<dyn Error>> {
    let path = get_pid_path();

    let msg = format!(
        "{} is not start, Please run '{} start' for start of {}",
        HIVE, HIVE, HIVE,
    )
    .color("red");
    let pid = if let Ok(pid) = std::fs::read_to_string(&path) {
        if pid.is_empty() {
            println!("{}", msg);
            return Ok(());
        }
        pid
    } else {
        println!("{}", msg);
        return Ok(());
    };

    let pid: i32 = pid.parse()?;
    unsafe {
        libc::kill(pid, 15 as libc::c_int);
    }
    std::fs::remove_file(path)?;
    println!("{}", format!("Stop {} of {}", HIVE, pid).color("green"));

    Ok(())
}

#[cfg(unix)]
pub fn replace_env<T: Into<String>>(val: T) -> Result<std::path::PathBuf, String> {
    let val = val.into();
    // 获取当前目录
    let current_dir = std::env::current_exe().or_else(|error| Err(format!("{}", error)))?;
    let current_dir: &Path = current_dir.parent().unwrap();
    debug!("{:?}", current_dir);
    let current_dir = current_dir.as_os_str();
    let current_dir = current_dir
        .to_str()
        .ok_or(format!("can't get path['{}'] as ASCII", val))?;
    return val
        .replace("${CWD}", current_dir)
        .parse()
        .or_else(|error| Err(format!("{}", error)));
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    setup_logger(&config.log_level)?;
    print_headers(&config)?;

    let mut runtime = Runtime::new()?;
    runtime.block_on(async move { start_server(config).await })
}

fn print_headers(config: &Config) -> Result<(), Box<dyn Error>> {
    let banner: String = r#"
  __     
 / _)_ _ 
/(_)(-(- "#
        .to_owned();
    println!(
        "{}    {}",
        banner.color("yellow"),
        PKG_VERSION.color("green")
    );
    println!("");
    info!(target: HIVE, "log level            {}", config.log_level);
    info!(target: HIVE, "--------------------------------");
    info!(
        target: HIVE,
        "{}",
        format!("listener on          {}:{} ...", config.ip, config.port).color("green")
    );
    info!(target: HIVE, "");
    Ok(())
}

async fn start_server(config: Config) -> Result<(), Box<dyn Error>> {
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
            if req.script != QUERY_NETWORK_STATES {
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
            if req.script != QUERY_NETWORK_STATES {
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

    let statement = if req.script == QUERY_NETWORK_STATES {
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
