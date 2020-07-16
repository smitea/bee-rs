use csv::ReaderBuilder;
use rusqlite::{
    functions::Context, types::Value as SqliteValue, Connection, Error as SQLiteError, Result,
    ToSql,
};
use std::panic::UnwindSafe;

/// Split with space for line.
///
/// ```sql
/// SELECT *FROM split_space('Hello world');
/// ```
/// => 'Hello','world'
///
fn split_space(context: &Context) -> Result<Vec<u8>> {
    let line = context.get::<String>(0)?;
    let cols = line
        .split_whitespace()
        .map(|val| val.to_string())
        .collect::<Vec<String>>();
    Ok(bincode::serialize(&cols)
        .or_else(|err| Err(SQLiteError::ModuleError(format!("{}", err))))?)
}

/// Split with CSV for line.
///
/// ```sql
/// SELECT *FROM split_space("'Hello','world'");
/// ```
/// => 'Hello','world'
///
fn split_csv(context: &Context) -> Result<Vec<u8>> {
    let line = context.get::<String>(0)?;

    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(line.as_bytes());

    let mut record = vec![];
    for result in rdr.records() {
        record = result
            .or_else(|err| Err(crate::Error::from(err)))?
            .iter()
            .map(|val| val.to_string())
            .collect::<Vec<String>>();
    }

    Ok(bincode::serialize(&record)
        .or_else(|err| Err(SQLiteError::ModuleError(format!("{}", err))))?)
}

/// Split with CSV for line.
///
/// ```sql
/// SELECT *FROM get("[csv,12]",1,'INT',0);
/// ```
/// => 'Hello','world'
///
fn get(context: &Context) -> Result<SqliteValue> {
    let output = context.get::<Vec<u8>>(0)?;
    let mut index = context.get::<i32>(1)?;
    let data_type = context.get::<String>(2)?;
    let default = context.get::<String>(3)?;

    let array: Vec<String> = bincode::deserialize(&output)
        .or_else(|err| Err(SQLiteError::ModuleError(format!("{}", err))))?;

    let len = array.len() as i32;
    if index < 0 {
        index = len + index - 1;
    }
    println!("output - {:?}",array);
    let value = array.get(index as usize).unwrap_or(&default);

    println!("GET - {},{},{},{}",value,index,data_type,default);
    let data_type = data_type.as_str();
    let value = match data_type {
        "INT" => {
            let value = value
                .parse::<i64>()
                .or_else(|err| Err(SQLiteError::ModuleError(format!("{}", err))))?;
            SqliteValue::Integer(value)
        }
        "REAL" => {
            let value = value
                .parse::<f64>()
                .or_else(|err| Err(SQLiteError::ModuleError(format!("{}", err))))?;
            SqliteValue::Real(value)
        }
        _ => SqliteValue::Text(value.clone()),
    };

    println!("output 2 - {:?}",value);
    Ok(value)
}

fn register_function<F, T>(db: &Connection, name: &str, args: i32, x_func: F) -> Result<()>
where
    F: FnMut(&Context<'_>) -> Result<T> + Send + UnwindSafe + 'static,
    T: ToSql,
{
    db.create_scalar_function(
        name,
        args,
        rusqlite::functions::FunctionFlags::default(),
        x_func,
    )?;
    Ok(())
}

pub fn init_functions(db: &Connection) -> Result<()> {
    register_function(&db, "split_space", 1, split_space)?;
    register_function(&db, "split_csv", 1, split_csv)?;
    register_function(&db, "get", 4, get)?;

    Ok(())
}
