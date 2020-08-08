#[macro_use]
extern crate prettytable;
use prettytable::{Cell, Row, Table};
use std::{
    env, io,
    io::BufRead,
    time::{Duration, SystemTime},
};
use bee_core::{Error, new_connection, Response};

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);
    let url: &str = args
        .get(1)
        .expect("must be a connection url, forexample: bee sqlite:agent:default");
    if url.trim().is_empty() {
        println!("must be a connection url, forexample: bee sqlite:agent:default");
        return;
    }
    println!("connecting {} ...", url);
    let connection = new_connection(url).unwrap();

    println!("Welcome to bee");
    println!("----------------------------------------");
    let mut buffer = String::new();
    loop {
        let mut line = String::new();
        let stdin = io::stdin();
        stdin.lock().read_line(&mut line).unwrap();
        buffer.push_str(&line);

        if line.trim() == "quit" || line.trim() == "exit" {
            println!("Bye bye.");
            break;
        }
        let last_char = line.trim().bytes().last().unwrap_or(0x00);
        if last_char == 0x3B {
            let now = SystemTime::now();
            match connection.new_statement(&buffer, Duration::from_secs(10)) {
                Ok(stat) => {
                    let resp = stat.wait();
                    match resp {
                        Ok(resp) => print_resp(resp),
                        Err(err) => handle_error(err),
                    }
                }
                Err(err) => handle_error(err),
            };
            println!("");
            let elapsed = now.elapsed().unwrap();
            println!("used {} ms.", elapsed.as_millis());
            println!("----------------------------------------");
            buffer = "".to_string();
        }
    }
}

fn handle_error(err: Error) {
    let mut table = Table::new();
    table.add_row(row!["code", "msg"]);
    table.add_row(row![&err.get_code().to_string(), err.get_msg()]);
    table.printstd();
}

fn print_resp(resp: Response) {
    let mut table = Table::new();

    let columns: Vec<Cell> = resp.columns().iter().map(|val| Cell::new(&val.0)).collect();
    table.add_row(Row::new(columns));
    for row in resp {
        match row {
            Ok(row) => {
                let row: Vec<Cell> = row.iter().map(|val| Cell::new(&val.to_string())).collect();
                table.add_row(Row::new(row));
            }
            Err(err) => handle_error(err),
        }
    }
    table.printstd();
}
