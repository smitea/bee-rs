use rusqlite::{
    functions::Context, types::Value as SqliteValue, Connection, Error as SQLiteError, Result,
    ToSql,
};
use std::panic::UnwindSafe;

fn split_space(context: &Context) -> Result<Vec<u8>> {
    let line = context.get::<String>(0)?;
    let cols = line
        .split_whitespace()
        .map(|val| val.to_string())
        .collect::<Vec<String>>();
    Ok(bincode::serialize(&cols)
        .or_else(|err| Err(SQLiteError::ModuleError(format!("{}", err))))?)
}

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

    let value = array.get(index as usize).unwrap_or(&default);

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
    register_function(&db, "get", 4, get)?;

    Ok(())
}
