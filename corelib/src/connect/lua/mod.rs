use crate::{
    new_req, new_req_none, Args, Configure, Connection, DataSource, Error, Request, Result, Value,
};
use parking_lot::RwLock;
use rlua::{Context, Lua, StdLib};
use std::{collections::HashMap, sync::Arc, time::Duration};

type CallFunc = dyn 'static + Send + Sync + Fn(&Args) -> Result<Value>;

mod convert;

pub struct LuaSession {
    ds_list: Arc<RwLock<HashMap<String, Arc<Box<dyn DataSource>>>>>,
    func_list: Arc<RwLock<HashMap<String, Arc<Box<CallFunc>>>>>,
}

impl LuaSession {
    pub fn new() -> LuaSession {
        LuaSession {
            ds_list: Arc::new(RwLock::new(HashMap::new())),
            func_list: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Configure for LuaSession {
    fn register_source(&self, ds: Box<dyn crate::DataSource>) -> crate::Result<()> {
        let mut lock = self
            .ds_list
            .try_write_for(Duration::from_secs(10))
            .ok_or(Error::lock_faild("lock timeout at 'mkdir'"))?;
        let _ = lock.insert(ds.name().to_owned(), Arc::new(ds));
        Ok(())
    }

    fn register_func<F, V: Into<Value>>(&self, name: &str, _: usize, func: F) -> crate::Result<()>
    where
        F: Fn(&crate::Args) -> crate::Result<V> + Send + Sync + std::panic::UnwindSafe + 'static,
    {
        let mut lock = self
            .func_list
            .try_write_for(Duration::from_secs(10))
            .ok_or(Error::lock_faild("lock timeout at 'mkdir'"))?;
        let _ = lock.insert(
            name.to_owned(),
            Arc::new(Box::new(move |args: &Args| {
                let value = func(args)?;
                Ok(value.into())
            })),
        );
        Ok(())
    }
}

impl Connection for LuaSession {
    fn new_statement(
        &self,
        script: &str,
        timeout: std::time::Duration,
    ) -> crate::Result<crate::Statement> {
        let (mut request, response) = new_req(Args::new(), timeout);
        let script = script.to_string();
        let ds_list = self.ds_list.clone();
        let func_list = self.func_list.clone();

        let lua = Lua::new_with(
            StdLib::BASE | StdLib::STRING | StdLib::UTF8 | StdLib::MATH | StdLib::TABLE,
        );
        let _ = async_std::task::spawn(async move {
            if let Err(err) = run_lua_script(lua, &mut request, script, ds_list, func_list) {
                let _ = request.error(err);
            } else {
                let _ = request.ok();
            }
            drop(request);
        });
        Ok(response)
    }
}

fn run_lua_script(
    lua: Lua,
    request: &mut Request,
    script: String,
    ds_list: Arc<RwLock<HashMap<String, Arc<Box<dyn DataSource>>>>>,
    func_list: Arc<RwLock<HashMap<String, Arc<Box<CallFunc>>>>>,
) -> Result<()> {
    let req = request.clone();
    lua.context(move |mut lua_context| {
        let script = script.clone();
        register_context(req, &mut lua_context, ds_list.clone(), func_list.clone())?;
        lua_context.load(&script).exec()
    })?;
    drop(lua);
    return Ok(());
}

fn register_context(
    request: Request,
    context: &mut Context,
    ds_list: Arc<RwLock<HashMap<String, Arc<Box<dyn DataSource>>>>>,
    func_list: Arc<
        RwLock<HashMap<String, Arc<Box<dyn Fn(&Args) -> Result<Value> + Send + Sync + 'static>>>>,
    >,
) -> Result<()> {
    let global = context.globals();

    let lock = ds_list
        .try_read_for(Duration::from_secs(10))
        .ok_or(Error::lock_faild("lock timeout at 'mkdir'"))?;
    for (key, ds) in lock.iter() {
        let ds = ds.clone();
        let function = context.create_function(move |_, args: Args| {
            let ds = ds.clone();
            let (mut request, statement) = new_req_none(args);
            let _ = async_std::task::spawn_blocking(move || {
                if let Err(err) = ds.collect(&mut request) {
                    let _ = request.error(err);
                } else {
                    let _ = request.ok();
                }
            });
            let response = statement.wait()?;
            Ok(response)
        })?;
        global.set(key.clone(), function)?;
    }

    let lock = func_list
        .try_read_for(Duration::from_secs(10))
        .ok_or(Error::lock_faild("lock timeout at 'mkdir'"))?;
    for (key, func) in lock.iter() {
        let func = func.clone();
        let function = context.create_function(move |_, args: Args| Ok(func(&args)?))?;
        global.set(key.clone(), function)?;
    }

    global.set("_request", request)?;
    Ok(())
}

#[test]
fn test() {
    let lua_script = r#"
        local resp=filesystem();
        while(resp:has_next())
        do
            _request:commit(_next);
        end
        "#;
    let conn = crate::new_connection("lua:agent:default").unwrap();

    let statement = conn
        .new_statement(lua_script, std::time::Duration::from_secs(2))
        .unwrap();
    let resp = statement.wait().unwrap();
    let cols = resp.columns();
    assert_eq!(5, cols.len());

    let mut index = 0;
    for row in resp {
        let _ = row.unwrap();
        index += 1;
    }
    assert!(index > 0);
}

#[test]
#[should_panic(expected = "runtime error:")]
fn test_no_such_func() {
    let lua_script = r#"
        local resp=test();
        while(resp:has_next())
        do
            _request:commit(_next);
        end
    "#;
    let conn = crate::new_connection("lua:agent:default").unwrap();

    let statement = conn
        .new_statement(lua_script, std::time::Duration::from_secs(2))
        .unwrap();
    let _ = statement.wait().unwrap();
}

#[test]
#[should_panic(expected = "runtime error:")]
fn test_runtime() {
    let lua_script = r#"
        local resp=test();
        while(resp:has_next())
        do
            _request:commit(io);
        end
        "#;
    let conn = crate::new_connection("lua:agent:default").unwrap();

    let statement = conn
        .new_statement(lua_script, std::time::Duration::from_secs(2))
        .unwrap();
    let _ = statement.wait().unwrap();
}
