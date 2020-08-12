#[macro_use]
extern crate prettytable;
extern crate liner;

use bee_codec::*;
use env::current_dir;
use futures;
use futures::SinkExt;
use liner::{Completer, Context, CursorPosition, Event, EventKind, FilenameCompleter, Prompt};
use prettytable::{format, Cell, Row, Table};
use std::{
    env, io,
    mem::replace,
    net::SocketAddr,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, SystemTime},
};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::stream::StreamExt;

fn highlight_dodo(s: &str) -> String {
    s.to_string()
}

struct NoCommentCompleter {
    inner: Option<FilenameCompleter>,
}

impl Completer for NoCommentCompleter {
    fn completions(&mut self, start: &str) -> Vec<String> {
        if let Some(inner) = &mut self.inner {
            inner.completions(start)
        } else {
            Vec::new()
        }
    }

    fn on_event<W: std::io::Write>(&mut self, event: Event<W>) {
        if let EventKind::BeforeComplete = event.kind {
            let (_, pos) = event.editor.get_words_and_cursor_position();

            let filename = match pos {
                CursorPosition::InWord(i) => i > 0,
                CursorPosition::InSpace(Some(_), _) => true,
                CursorPosition::InSpace(None, _) => false,
                CursorPosition::OnWordLeftEdge(i) => i >= 1,
                CursorPosition::OnWordRightEdge(i) => i >= 1,
            };

            if filename {
                let completer = FilenameCompleter::new(Some(current_dir().unwrap()));
                let _ = replace(&mut self.inner, Some(completer));
            } else {
                let _ = replace(&mut self.inner, None);
            }
        }
    }
}

enum ScriptCommand<'a> {
    Use(&'a str),
    Script(&'a str),
    Exit,
}

#[tokio::main]
async fn main() {
    let mut ctx = Context::new();
    let mut completer = NoCommentCompleter { inner: None };

    let version = env!("CARGO_PKG_VERSION").to_string();
    let hostname = get_hostname().unwrap();
    let addr = get_arg_uri();

    println!("Bee shell version {}, hostname: {}", version, hostname);
    let mut buffer = String::new();
    let mut connection: Option<(
        FramedWrite<OwnedWriteHalf, PacketCodec>,
        FramedRead<OwnedReadHalf, PacketCodec>,
    )> = None;

    let req_id = AtomicUsize::new(0);
    loop {
        let (line, is_exit) = read_line(&mut ctx, &mut completer);
        if is_exit {
            break;
        }
        buffer.push_str(&line.trim());
        buffer.push('\n');
        if is_end(&line) {
            let buf = buffer.trim().to_owned();
            let buf = &buf[0..(buf.len() - 1)];
            match handler_line(&buf) {
                ScriptCommand::Use(url) => {
                    if url.is_empty() {
                        println!("url is empty");
                    } else {
                        match new_connection(&addr, &hostname, url).await {
                            Ok(conn) => {
                                // 释放旧的连接
                                if let Some(old_conn) = connection {
                                    drop(old_conn);
                                }
                                connection = Some(conn);
                            }
                            Err(err) => {
                                println!(
                                    "connected to {} is failed: BEE-{}[{}]",
                                    url,
                                    err.get_code(),
                                    err.get_msg()
                                );
                            }
                        }
                    }
                }
                ScriptCommand::Script(script) => {
                    if let Some((writer, reader)) = &mut connection {
                        let id = req_id.fetch_add(1, Ordering::Release);
                        if let Err(err) = printf_statement(
                            id as u32,
                            script,
                            Duration::from_secs(10),
                            writer,
                            reader,
                        )
                        .await
                        {
                            println!(
                                "request statement is failed: BEE-{}[{}]",
                                err.get_code(),
                                err.get_msg()
                            )
                        }
                    } else {
                        println!("not connected.");
                    }
                }
                ScriptCommand::Exit => {
                    break;
                }
            }
            buffer = "".to_string();
        }
    }

    println!("Bye bye");
    ctx.history.commit_to_file();
}

async fn new_connection(
    addr: &SocketAddr,
    host_name: &str,
    url: &str,
) -> Result<(
    FramedWrite<OwnedWriteHalf, PacketCodec>,
    FramedRead<OwnedReadHalf, PacketCodec>,
)> {
    let stream = TcpStream::connect(addr).await?;
    stream.set_nodelay(true)?;
    let (reader, writer) = stream.into_split();
    let mut reader_framed = FramedRead::new(reader, PacketCodec);
    let mut writer_framed = FramedWrite::new(writer, PacketCodec);

    // 连接数据源
    connect_ds(host_name, url, &mut writer_framed, &mut reader_framed).await?;
    println!("Connected to {} ", url);

    Ok((writer_framed, reader_framed))
}

fn get_arg_uri() -> SocketAddr {
    let args: Vec<String> = env::args().collect();
    let url: &str = args
        .get(1)
        .expect("must be a connection url, forexample: bee sqlite:agent:default");
    if url.trim().is_empty() {
        panic!("must be a connection url, forexample: bee sqlite:agent:default");
    }
    return url.parse().unwrap();
}

fn get_hostname() -> Result<String> {
    let hostname = format!("{}", hostname::get()?.to_string_lossy());
    Ok(hostname)
}

fn handler_line(line: &str) -> ScriptCommand {
    if line.ends_with("quit") || line.ends_with("exit") {
        ScriptCommand::Exit
    } else if line.starts_with("use") {
        ScriptCommand::Use(line.split("use").last().unwrap_or("").trim())
    } else {
        ScriptCommand::Script(line)
    }
}

fn can_exit(err: std::io::Error) -> bool {
    match err.kind() {
        io::ErrorKind::Interrupted => false,
        io::ErrorKind::UnexpectedEof => true,
        _ => panic!("error: {:?}", err),
    }
}

fn is_end(line: &str) -> bool {
    let last_char = line.trim().bytes().last().unwrap_or(0x00);
    return last_char == 0x3B;
}

fn read_line(ctx: &mut Context, completer: &mut NoCommentCompleter) -> (String, bool) {
    match ctx.read_line(
        Prompt::from("> "),
        Some(Box::new(highlight_dodo)),
        completer,
    ) {
        Ok(line) => (line, false),
        Err(err) => ("".to_owned(), can_exit(err)),
    }
}

async fn connect_ds(
    host_name: &str,
    url: &str,
    writer_framed: &mut FramedWrite<OwnedWriteHalf, PacketCodec>,
    read: &mut FramedRead<OwnedReadHalf, PacketCodec>,
) -> Result<()> {
    write_connection_resp(host_name, url, writer_framed).await?;
    read_connection_resp(read).await?;
    Ok(())
}

async fn write_connection_resp(
    host_name: &str,
    url: &str,
    writer_framed: &mut FramedWrite<OwnedWriteHalf, PacketCodec>,
) -> Result<()> {
    writer_framed
        .send(Packet::ConnectReq(ConnectionReq {
            url: url.to_string(),
            application: host_name.to_string(),
        }))
        .await?;

    Ok(())
}

async fn read_connection_resp(read: &mut FramedRead<OwnedReadHalf, PacketCodec>) -> Result<()> {
    if let Some(Ok(Packet::ConnectResp(resp))) = read.next().await {
        if let ConnectionResp::Error(err) = resp {
            return Err(err);
        }
    }
    Ok(())
}

async fn new_statement(
    id: u32,
    script: &str,
    timeout: Duration,
    writer_framed: &mut FramedWrite<OwnedWriteHalf, PacketCodec>,
    read: &mut FramedRead<OwnedReadHalf, PacketCodec>,
) -> Result<Table> {
    write_statement(id, script, timeout, writer_framed).await?;
    read_statement(id, read).await
}

async fn write_statement(
    id: u32,
    script: &str,
    timeout: Duration,
    writer_framed: &mut FramedWrite<OwnedWriteHalf, PacketCodec>,
) -> Result<()> {
    let req = Packet::StatementReq(StatementReq {
        id,
        script: script.to_string(),
        timeout: timeout.as_secs() as u32,
    });
    writer_framed.send(req).await?;
    Ok(())
}

async fn read_statement(
    id: u32,
    read: &mut FramedRead<OwnedReadHalf, PacketCodec>,
) -> Result<Table> {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    while let Some(Ok(Packet::StatementResp(resp))) = read.next().await {
        if id != resp.id {
            return Err(Error::other(-1, "network error!"));
        }
        match resp.state {
            StatementStateResp::Columns(cols) => {
                table.set_titles(Row::new(cols.iter().map(|val| Cell::new(&val.0)).collect()));
            }
            StatementStateResp::Row(row) => {
                table.add_row(row.iter().map(|val| Cell::new(&val.to_string())).collect());
            }
            StatementStateResp::Abort => {
                break;
            }
            StatementStateResp::Error(err) => {
                table.add_row(row!["code", "msg"]);
                table.add_row(row![&err.get_code().to_string(), err.get_msg()]);
                break;
            }
        }
    }

    return Ok(table);
}

async fn printf_statement(
    id: u32,
    script: &str,
    timeout: Duration,
    writer_framed: &mut FramedWrite<OwnedWriteHalf, PacketCodec>,
    read: &mut FramedRead<OwnedReadHalf, PacketCodec>,
) -> Result<()> {
    let now = SystemTime::now();
    println!("");
    let table = new_statement(id, script, timeout, writer_framed, read).await?;
    table.printstd();
    println!("");
    let elapsed = now.elapsed().unwrap();
    println!("used {} ms.", elapsed.as_millis());
    println!("");

    return Ok(());
}
