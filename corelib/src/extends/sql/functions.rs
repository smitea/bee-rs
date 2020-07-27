use csv::ReaderBuilder;
use rusqlite::{
    functions::Context, types::Value as SqliteValue, Connection, Error as SQLiteError, Result,
    ToSql,
};
use std::panic::UnwindSafe;

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