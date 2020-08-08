use crate::{
    new_req, new_req_none, Args, Configure, Connection, DataSource, Request, Result, Value,
};
use async_std::task;
use rlua::{Context, Lua};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

type CallFunc = dyn 'static + Send + Sync + Fn(&Args) -> Result<Value>;

mod convert;

pub struct LuaSession {
    ds_list: Arc<Mutex<HashMap<String, Arc<Box<dyn DataSource>>>>>,
    func_list: Arc<Mutex<HashMap<String, Arc<Box<CallFunc>>>>>,
}

impl LuaSession {
    pub fn new() -> LuaSession {
        LuaSession {
            ds_list: Arc::new(Mutex::new(HashMap::new())),
            func_list: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Configure for LuaSession {
    fn register_source(&self, ds: Box<dyn crate::DataSource>) -> crate::Result<()> {
        let mut lock = self.ds_list.lock()?;
        let _ = lock.insert(ds.name().to_owned(), Arc::new(ds));
        Ok(())
    }

    fn register_func<F, V: Into<Value>>(&self, name: &str, _: usize, func: F) -> crate::Result<()>
    where
        F: Fn(&crate::Args) -> crate::Result<V> + Send + Sync + std::panic::UnwindSafe + 'static,
    {
        let mut lock = self.func_list.lock()?;
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
        let (request, response) = new_req(Args::new(), timeout);

        let lua = Lua::new();
        lua.context(|mut lua_context| {
            register_context(
                request,
                &mut lua_context,
                self.ds_list.clone(),
                self.func_list.clone(),
            )?;
            lua_context.load(script).exec()
        })?;
        Ok(response)
    }
}

fn register_context(
    request: Request,
    context: &mut Context,
    ds_list: Arc<Mutex<HashMap<String, Arc<Box<dyn DataSource>>>>>,
    func_list: Arc<
        Mutex<HashMap<String, Arc<Box<dyn Fn(&Args) -> Result<Value> + Send + Sync + 'static>>>>,
    >,
) -> Result<()> {
    let global = context.globals();

    let lock = ds_list.lock()?;
    for (key, ds) in lock.iter() {
        let ds = ds.clone();
        let function = context.create_function(move |_, args: Args| {
            let ds = ds.clone();
            let (mut request, statement) = new_req_none(args);
            let _ = task::spawn(async move {
                if let Err(err) = ds.collect(&mut request) {
                    let _ = request.error(err);
                }
            });
            let response = statement.wait()?;
            Ok(response)
        })?;
        global.set(key.clone(), function)?;
    }

    let lock = func_list.lock()?;
    for (key, func) in lock.iter() {
        let func = func.clone();
        let function = context.create_function(move |_, args: Args| Ok(func(&args)?))?;
        global.set(key.clone(), function)?;
    }

    global.set("_request", request)?;
    Ok(())
}
