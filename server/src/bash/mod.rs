use bee_core::{Promise, Result, DataSource, Columns, ToData, Row, Driver, Instance, Connection, Value, Args, Error, Request};
use std::time::Duration;
use csv::ReaderBuilder;

#[cfg(feature = "agent")]
mod local;
#[cfg(feature = "remote")]
mod remote;

#[derive(Data)]
pub struct BashRow {
    line: String,
    line_num: u32,
}

impl BashRow {
    pub fn new<T: Into<String>>(line: T, line_num: usize) -> Self {
        Self {
            line: line.into(),
            line_num: line_num as u32,
        }
    }
}

pub trait Bash: Send + Sync {
    fn run_cmd(&self, script: &str, timeout: Duration, promise: &mut Promise<BashRow>) -> Result<()>;
}

pub struct BashWrapper(Box<dyn Bash>);

impl DataSource for BashWrapper {
    fn name(&self) -> &str {
        "bash"
    }

    fn args(&self) -> Columns {
        columns![String: "script", Integer: "timeout", String: "encode"]
    }

    fn columns(&self) -> Columns {
        BashRow::columns()
    }

    fn collect(&self, request: &mut Request) -> Result<()> {
        let mut promise: Promise<BashRow> = request.head()?;
        let arg = promise.get_args();
        let script: String = arg.get(0)?;
        let timeout: u32 = arg.get(1).unwrap_or(10);
        self.0.run_cmd(&script, Duration::from_millis(timeout as u64), &mut promise)
    }
}

pub struct BashDriver;

impl BashDriver {
    fn create_datasource(instance: Instance, ds: &str) -> Result<Box<dyn DataSource>> {
        let data_source: Box<dyn Bash> = match ds {
            #[cfg(feature = "agent")]
            "local" => local::new_local_bash(instance)?,
            #[cfg(feature = "remote")]
            "remote" => remote::new_remote_bash(instance)?,
            _ => return Err(Error::index_param("ds"))
        };

        return Ok(Box::new(BashWrapper(data_source)));
    }
}

/// Split with space for line.
///
/// ```sql
/// SELECT *FROM split_space('Hello world');
/// ```
/// => 'Hello','world'
///
fn split_space(args: &Args) -> Result<Vec<u8>> {
    let line = args.get::<String>(0)?;
    let cols = line
        .split_whitespace()
        .map(|val| val.to_string())
        .collect::<Vec<String>>();
    bincode::serialize(&cols).or_else(|err| Err(Error::invalid_type(err.to_string())))
}

/// Split with CSV for line.
///
/// ```sql
/// SELECT *FROM split_space("'Hello','world'");
/// ```
/// => 'Hello','world'
///
fn split_csv(args: &Args) -> Result<Vec<u8>> {
    let line = args.get::<String>(0)?;

    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(line.as_bytes());

    let mut record = vec![];
    for result in rdr.records() {
        record = result
            .or_else(|err| Err(Error::invalid_type(err.to_string())))?
            .iter()
            .map(|val| val.to_string())
            .collect::<Vec<String>>();
    }

    bincode::serialize(&record).or_else(|err| Err(Error::invalid_type(err.to_string())))
}

/// Split with CSV for line.
///
/// ```sql
/// SELECT *FROM get("[csv,12]",1,'INT',0);
/// ```
/// => 'Hello','world'
///
fn get(args: &Args) -> Result<Value> {
    let output = args.get::<Vec<u8>>(0)?;
    let mut index = args.get::<i32>(1)?;
    let data_type = args.get::<String>(2).unwrap_or("TEXT".to_owned());

    // 为动态类型
    let default = args.get::<String>(3)?;

    let array: Vec<String> = bincode::deserialize(&output)
        .or_else(|err| Err(Error::other(0, format!("{}", err))))?;

    let len = array.len() as i32;
    if index < 0 {
        index = len + index - 1;
    }
    let value = array.get(index as usize).unwrap_or(&default);
    let data_type = data_type.as_str();

    parse_value(data_type, value)
}

fn parse_value(data_type: &str, value: &String) -> Result<Value> {
    let value = match data_type {
        "INT" => {
            let value = value.parse::<i64>()?;
            Value::Integer(value)
        }
        "REAL" => {
            let value = value.parse::<f64>()?;
            Value::Number(value)
        }
        _ => Value::String(value.clone()),
    };

    Ok(value)
}

impl Driver for BashDriver {
    fn name(&self) -> &str {
        "bash"
    }

    fn new_datasource(&self, instance: Instance) -> Result<Box<dyn DataSource>> {
        let ds = instance.get_param("ds").unwrap_or("local".to_owned());
        Self::create_datasource(instance, &ds)
    }
}

pub fn new_driver<T: Connection>(connect: T) -> Result<Box<dyn Driver>> {
    connect.register_func(get)?;
    connect.register_func(split_csv)?;
    connect.register_func(split_space)?;

    Ok(Box::new(BashDriver))
}