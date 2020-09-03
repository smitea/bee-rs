use crate::{new_req, Args, Configure, Connection, DataSource, Request, Result, Value};
use rlua::{Context, Lua, StdLib};
use std::{collections::HashMap, sync::Arc};

type CallFunc = dyn 'static + Send + Sync + Fn(&Args) -> Result<Value>;

mod convert;

pub struct LuaSession {
    ds_list: HashMap<String, Arc<Box<dyn DataSource>>>,
    func_list: HashMap<String, Arc<Box<CallFunc>>>,
}

impl LuaSession {
    pub fn new() -> LuaSession {
        LuaSession {
            ds_list: HashMap::new(),
            func_list: HashMap::new(),
        }
    }
}

impl Configure for LuaSession {
    fn register_source(&mut self, ds: Box<dyn crate::DataSource>) -> crate::Result<()> {
        let _ = self.ds_list.insert(ds.name().to_owned(), Arc::new(ds));
        Ok(())
    }

    fn register_func<F, V: Into<Value>>(
        &mut self,
        name: &str,
        _: usize,
        func: F,
    ) -> crate::Result<()>
    where
        F: Fn(&crate::Args) -> crate::Result<V> + Send + Sync + std::panic::UnwindSafe + 'static,
    {
        let _ = self.func_list.insert(
            name.to_owned(),
            Arc::new(Box::new(move |args: &Args| {
                let value = func(args)?;
                Ok(value.into())
            })),
        );
        Ok(())
    }
}

#[async_trait]
impl Connection for LuaSession {
    async fn new_statement(&self, script: &str) -> crate::Result<crate::Statement> {
        let (mut request, response) = new_req(Args::new());
        let script = script.to_string();
        let ds_list = self.ds_list.clone();
        let func_list = self.func_list.clone();

        let _ = smol::spawn(async move {
            println!("collecting [{}]", script);
            if let Err(err) = run_lua_script(&mut request, script, ds_list, func_list) {
                let _ = request.error(err);
            } else {
                let _ = request.ok();
            }
            println!("collected");
        })
        .detach();
        Ok(response)
    }
}

fn run_lua_script(
    request: &mut Request,
    script: String,
    ds_list: HashMap<String, Arc<Box<dyn DataSource>>>,
    func_list: HashMap<String, Arc<Box<CallFunc>>>,
) -> Result<()> {
    let lua = new_lua();
    let req = request.clone();
    lua.context(move |mut lua_context| {
        let script = script.clone();
        register_context(req, &mut lua_context, ds_list, func_list)?;
        lua_context.load(&script).exec()
    })?;
    drop(lua);
    return Ok(());
}

fn new_lua() -> Lua {
    Lua::new_with(StdLib::BASE | StdLib::STRING | StdLib::UTF8 | StdLib::MATH | StdLib::TABLE)
}

fn register_context(
    request: Request,
    context: &mut Context,
    ds_list: HashMap<String, Arc<Box<dyn DataSource>>>,
    func_list: HashMap<String, Arc<Box<dyn Fn(&Args) -> Result<Value> + Send + Sync + 'static>>>,
) -> Result<()> {
    let global = context.globals();
    for (key, ds) in ds_list.iter() {
        let ds: Arc<Box<dyn DataSource>> = ds.clone();
        let function = context.create_function(move |_, args: Args| {
            let ds: Arc<Box<dyn DataSource>> = ds.clone();
            let reg = ds.get_register();
            let ex: Arc<smol::Executor> = reg.get_state();
            let (mut request, statement) = new_req(args);
            let _ = ex
                .spawn(async move {
                    if let Err(err) = ds.collect(&mut request).await {
                        let _ = request.error(err);
                    } else {
                        let _ = request.ok();
                    }
                })
                .detach();
            let response = smol::block_on(async move { statement.wait().await })?;
            Ok(response)
        })?;
        global.set(key.clone(), function)?;
    }

    for (key, func) in func_list.iter() {
        let func = func.clone();
        let function = context.create_function(move |_, args: Args| Ok(func(&args)?))?;
        global.set(key.clone(), function)?;
    }

    global.set("_request", request)?;
    Ok(())
}

#[test]
fn test() {
    smol::block_on(async {
        let lua_script = r#"
        local resp=filesystem();
        while(resp:has_next())
        do
            _request:commit(_next);
        end
        "#;
        let conn = crate::new_connection("lua:agent:default").await.unwrap();

        let statement = conn.new_statement(lua_script).await.unwrap();
        let resp = statement.wait().await.unwrap();
        let cols = resp.columns();
        assert_eq!(5, cols.len());

        let mut index = 0;
        for row in resp {
            let _ = row.unwrap();
            index += 1;
        }
        assert!(index > 0);
    });
}

#[test]
#[should_panic(expected = "runtime error:")]
fn test_no_such_func() {
    smol::block_on(async {
        let lua_script = r#"
        local resp=test();
        while(resp:has_next())
        do
            _request:commit(_next);
        end
    "#;
        let conn = crate::new_connection("lua:agent:default").await.unwrap();

        let statement = conn.new_statement(lua_script).await.unwrap();
        let _ = statement.wait().await.unwrap();
    });
}

#[test]
#[should_panic(expected = "runtime error:")]
fn test_runtime() {
    smol::block_on(async {
        let lua_script = r#"
        local resp=test();
        while(resp:has_next())
        do
            _request:commit(io);
        end
        "#;
        let conn = crate::new_connection("lua:agent:default").await.unwrap();

        let statement = conn.new_statement(lua_script).await.unwrap();
        let _ = statement.wait().await.unwrap();
    });
}
