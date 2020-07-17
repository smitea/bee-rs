use crate::{Error, Value, CODE};
use rusqlite::{
    types::{FromSql, FromSqlError, Value as SqliteValue, ValueRef},
    Error as SQLiteError, ToSql,
};

const BASE_CODE: i32 = 240;
const SQLITESINGLETHREADEDMODE: i32 = CODE!(BASE_CODE, 0);
const FROMSQLCONVERSIONFAILURE: i32 = CODE!(BASE_CODE, 1);
const INTEGRALVALUEOUTOFRANGE: i32 = CODE!(BASE_CODE, 2);
const UTF8ERROR: i32 = CODE!(BASE_CODE, 3);
const NULERROR: i32 = CODE!(BASE_CODE, 4);
const INVALIDPARAMETERNAME: i32 = CODE!(BASE_CODE, 5);
const INVALIDPATH: i32 = CODE!(BASE_CODE, 6);
const EXECUTERETURNEDRESULTS: i32 = CODE!(BASE_CODE, 7);
const QUERYRETURNEDNOROWS: i32 = CODE!(BASE_CODE, 8);
const INVALIDCOLUMNINDEX: i32 = CODE!(BASE_CODE, 9);
const INVALIDCOLUMNNAME: i32 = CODE!(BASE_CODE, 10);
const INVALIDCOLUMNTYPE: i32 = CODE!(BASE_CODE, 11);
const STATEMENTCHANGEDROWS: i32 = CODE!(BASE_CODE, 12);
const TOSQLCONVERSIONFAILURE: i32 = CODE!(BASE_CODE, 13);
const INVALIDQUERY: i32 = CODE!(BASE_CODE, 14);
const MULTIPLESTATEMENT: i32 = CODE!(BASE_CODE, 15);
const USERFUNCTIONERROR: i32 = CODE!(BASE_CODE, 16);
const INVALIDFUNCTIONPARAMETERTYPE: i32 = CODE!(BASE_CODE, 17);
const UNWINDINGPANIC: i32 = CODE!(BASE_CODE, 18);
const GETAUXWRONGTYPE: i32 = CODE!(BASE_CODE, 19);
const INVALIDPARAMETERCOUNT: i32 = CODE!(BASE_CODE, 20);
const OTHER: i32 = CODE!(BASE_CODE, 21);
pub(crate) const INVALIDCOLUMNCOUNT: i32 = CODE!(BASE_CODE, 22);

impl From<SQLiteError> for Error {
    fn from(err: SQLiteError) -> Self {
        let msg = err.to_string();
        match msg.parse::<Error>(){
            Ok(err) => err,
            Err(_) => {
                match err{
                    SQLiteError::SqliteFailure(code, msg) => Error::other(code.extended_code, msg.unwrap_or("".to_owned())),
                    SQLiteError::SqliteSingleThreadedMode => Error::other(SQLITESINGLETHREADEDMODE,"sqlite is single of thread mode"),
                    SQLiteError::FromSqlConversionFailure(index, d, err) => Error::other(FROMSQLCONVERSIONFAILURE,format!("failed to SQL parser at column[{}] and type[{}] ,the reason : {}", index, d, err)),
                    SQLiteError::IntegralValueOutOfRange(index, value) => Error::other(INTEGRALVALUEOUTOFRANGE,format!("integral value is out of range at column[{}] and value[{}]",index,value)), 
                    SQLiteError::Utf8Error(err) => Error::other(UTF8ERROR,err),
                    SQLiteError::NulError(err) => Error::other(NULERROR,err),
                    SQLiteError::InvalidParameterName(name) => Error::other(INVALIDPARAMETERNAME,format!("invalid params[{}]", name)),
                    SQLiteError::InvalidPath(path) => Error::other(INVALIDPATH,format!("invalid path: {:?}",path)), 
                    SQLiteError::ExecuteReturnedResults => Error::other(EXECUTERETURNEDRESULTS,format!("has a `execute` call returns rows")),
                    SQLiteError::QueryReturnedNoRows =>  Error::other(QUERYRETURNEDNOROWS,format!("has a query that was expected to return at least one row (e.g.,for `query_row`) did not return any")),
                    SQLiteError::InvalidColumnIndex(index) =>  Error::other(INVALIDCOLUMNINDEX,format!("invalid columnIndex[{}]",index)),
                    SQLiteError::InvalidColumnName(name) =>  Error::other(INVALIDCOLUMNNAME,format!("invalid columnName[{}]",name)),
                    SQLiteError::InvalidColumnType(index, name, d) =>  Error::other(INVALIDCOLUMNTYPE,format!("invalid columnType[{}] at {} and type: {:?}",name,index, d)),
                    SQLiteError::StatementChangedRows(rows) =>  Error::other(STATEMENTCHANGEDROWS,format!("has a query that was expected to insert one row did not insert {} rows",rows)),
                    SQLiteError::ToSqlConversionFailure(err) =>  Error::other(TOSQLCONVERSIONFAILURE,err),
                    SQLiteError::InvalidQuery =>  Error::other(INVALIDQUERY,format!("SQL is not a `SELECT`, is not read-only")),
                    SQLiteError::MultipleStatement =>  Error::other(MULTIPLESTATEMENT,format!("SQL contains multiple statements")),
                    SQLiteError::UserFunctionError(err) => Error::other(USERFUNCTIONERROR,err),
                    SQLiteError::InvalidFunctionParameterType(index, d) => Error::other(INVALIDFUNCTIONPARAMETERTYPE,format!("The params[{}] must be a {}",index,d)),
                    SQLiteError::UnwindingPanic => Error::other(UNWINDINGPANIC,"UnwindingPanic"),
                    SQLiteError::GetAuxWrongType => Error::other(GETAUXWRONGTYPE,"GetAuxWrongType"),
                    SQLiteError::InvalidParameterCount(index, count) => Error::other(INVALIDPARAMETERCOUNT,format!("invalid params[{}] and count = {}", index,count )),
                    _ => Error::other(OTHER,"")
                }
            }
        }
    }
}

impl From<Error> for SQLiteError {
    fn from(err: Error) -> Self {
        rusqlite::Error::ModuleError(err.to_string())
    }
}

impl ToSql for Value {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        let val = match self {
            Value::String(value) => {
                rusqlite::types::ToSqlOutput::Owned(SqliteValue::Text(value.clone()))
            }
            Value::Integer(value) => {
                rusqlite::types::ToSqlOutput::Owned(SqliteValue::Integer(value.clone()))
            }
            Value::Number(value) => rusqlite::types::ToSqlOutput::Owned(SqliteValue::Real(*value)),
            Value::Boolean(value) => {
                rusqlite::types::ToSqlOutput::Owned(SqliteValue::Integer(if *value {
                    1
                } else {
                    0
                }))
            }
            Value::Bytes(value) => {
                rusqlite::types::ToSqlOutput::Owned(SqliteValue::Blob(value.clone()))
            }
            Value::Nil => rusqlite::types::ToSqlOutput::Owned(SqliteValue::Null),
        };
        return Ok(val);
    }
}

impl FromSql for Value {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let val = match value {
            ValueRef::Null => Value::Nil,
            ValueRef::Integer(val) => Value::Integer(val),
            ValueRef::Real(val) => Value::Number(val),
            ValueRef::Text(val) => Value::String(
                String::from_utf8(Vec::from(val))
                    .or_else(|err| Err(FromSqlError::Other(Box::new(err))))?,
            ),
            ValueRef::Blob(val) => Value::Bytes(Vec::from(val)),
        };
        return Ok(val);
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Self {
        Error::invalid_type(err.to_string())
    }
}
