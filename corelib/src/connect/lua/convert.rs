use crate::{value::Bytes, Args, Error, Request, Response, Value};
use rlua::{FromLuaMulti, Table, UserData};
use std::{cell::Ref, convert::TryFrom};

const BASE_CODE: i32 = 241;

struct BytesWrapper(usize, Bytes);

type LuaError = rlua::Error;

impl From<LuaError> for Error {
    fn from(err: LuaError) -> Self {
        Error::other(BASE_CODE, err.to_string())
    }
}

impl From<Error> for LuaError {
    fn from(err: Error) -> Self {
        LuaError::RuntimeError(format!("{:?}", err))
    }
}

impl<'a> rlua::ToLua<'a> for Value {
    fn to_lua(self, lua: rlua::Context) -> Result<rlua::Value, LuaError> {
        let val = match self {
            Value::Boolean(val) => rlua::Value::Boolean(val),
            Value::String(val) => rlua::Value::String(lua.create_string(&val)?),
            Value::Integer(val) => rlua::Value::Integer(val),
            Value::Number(val) => rlua::Value::Number(val),
            Value::Bytes(val) => {
                let data = lua.create_userdata(BytesWrapper(val.len(), val))?;
                rlua::Value::UserData(data)
            }
            Value::Nil => rlua::Value::Nil,
        };
        Ok(val)
    }
}

impl<'a> FromLuaMulti<'a> for Args {
    fn from_lua_multi(values: rlua::MultiValue<'a>, _: rlua::Context<'a>) -> rlua::Result<Self> {
        let mut args = Args::new();
        for val in values {
            args.push(Value::try_from(val)?);
        }
        return Ok(args);
    }
}

impl<'a> TryFrom<rlua::Value<'a>> for Value {
    type Error = crate::error::Error;
    fn try_from(value: rlua::Value<'a>) -> Result<Self, Self::Error> {
        match value {
            rlua::Value::Boolean(value) => Ok(Value::Boolean(value)),
            rlua::Value::Function(func) => Err(crate::error::Error::invalid_type(format!(
                "not support lua function: {:?} to value",
                func
            ))),
            rlua::Value::Error(err) => Err(crate::error::Error::other(0x01, format!("{}", err))),
            rlua::Value::Integer(value) => Ok(Value::Integer(value)),
            rlua::Value::LightUserData(value) => Err(crate::error::Error::invalid_type(format!(
                "not support lua 'LightUserData': {:?} to value",
                value
            ))),
            rlua::Value::Nil => Ok(Value::Nil),
            rlua::Value::Number(value) => Ok(Value::Number(value)),
            rlua::Value::String(value) => Ok(Value::String(
                value
                    .to_str()
                    .or_else(|err| Err(crate::error::Error::invalid_type(format!("{}", err))))?
                    .to_owned(),
            )),
            rlua::Value::Table(value) => Err(crate::error::Error::invalid_type(format!(
                "not support lua 'Thread': {:?} to value",
                value
            ))),
            rlua::Value::Thread(value) => Err(crate::error::Error::invalid_type(format!(
                "not support lua 'Thread': {:?} to value",
                value
            ))),
            rlua::Value::UserData(value) => {
                if value.is::<BytesWrapper>() {
                    let wrapper: Ref<BytesWrapper> = value.borrow()?;
                    Ok(Value::Bytes(wrapper.1.clone()))
                } else {
                    Err(crate::error::Error::invalid_type(format!(
                        "not support lua 'Thread': {:?} to value",
                        value
                    )))
                }
            }
        }
    }
}

impl UserData for Response {
    fn add_methods<'lua, T: rlua::UserDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method_mut("has_next", |ctx, data, ()| {
            if let Some(row) = data.next_row() {
                let row = row?;
                let columns = data.columns();
                let next = ctx.create_table()?;

                for (i, (name, _)) in columns.iter().enumerate() {
                    let value = row.get_value(i)?;
                    next.set(name.clone(), value.clone())?;
                }

                ctx.globals().set("_next", next)?;
                Ok(true)
            } else {
                Ok(false)
            }
        });
    }
}

impl UserData for Request {
    fn add_methods<'lua, T: rlua::UserDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("error", |_, data, (code, msg): (i32, String)| {
            data.error(Error::other(code, msg))?;
            Ok(())
        });

        methods.add_method_mut("commit", |_, data, row: Table| {
            let values = row
                .pairs()
                .collect::<rlua::Result<Vec<(std::string::String, rlua::Value)>>>()?;

            let mut new_values = vec![];
            for (name, value) in values {
                new_values.push((name, Value::try_from(value)?));
            }
            data.commit(new_values)?;
            Ok(())
        });
    }
}

impl UserData for BytesWrapper {}

#[test]
fn test_tryfrom_value() {
    let lua = rlua::Lua::new();
    lua.context(move |lua_context| {
        let _ = Value::try_from(rlua::Value::Boolean(false)).unwrap();
        if let Ok(func) = lua_context.create_function(|_, _: String| Ok(())) {
            let rs = Value::try_from(rlua::Value::Function(func));
            assert!(rs.is_err());
        }

        if let Ok(func) = lua_context.create_function(|_, _: String| Ok(())) {
            assert!(Value::try_from(rlua::Value::Thread(
                lua_context.create_thread(func).unwrap()
            ))
            .is_err());
        }

        if let Ok(data) = lua_context.create_userdata(BytesWrapper(2, vec![0x01, 0x02])) {
            assert!(Value::try_from(rlua::Value::UserData(data)).is_ok());
        }

        let (req, _) = crate::new_req_none(Args::new());
        if let Ok(data) = lua_context.create_userdata(req) {
            assert!(Value::try_from(rlua::Value::UserData(data)).is_err());
        }

        let err = Value::try_from(rlua::Value::Error(rlua::Error::BindError));
        assert!(err.is_err());

        assert!(Value::try_from(rlua::Value::Integer(10)).is_ok());
        assert!(Value::try_from(rlua::Value::Table(lua_context.globals())).is_err());
    });
}
