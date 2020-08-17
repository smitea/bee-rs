use crate::{Error, Result, Value};
use std::{collections::HashMap, convert::TryFrom, str::FromStr};
use url::Url;

/// 连接的实例信息
#[derive(Debug, Clone)]
pub struct Instance {
    sess_mode: String,
    ds_mode: String,
    connect_mode: String,
    host: Option<String>,
    port: Option<u16>,
    resource: Option<String>,
    username: Option<String>,
    password: Option<String>,
    params: HashMap<String, Value>,
}

impl Instance {
    /// 根据 URI 格式来创建 `Instance`
    pub fn from(url: &str) -> Result<Self> {
        let protocols: Vec<&str> = url.splitn(3, ':').collect();
        let sess_mode: &str = protocols.get(0).ok_or(Error::index_param("session_mode"))?;
        let ds_mode: &str = protocols.get(1).ok_or(Error::index_param("ds_mode"))?;
        let url = protocols.get(2);

        let mut connect_mode = "default".to_owned();
        let mut params: HashMap<String, Value> = HashMap::new();
        let mut username = Option::None;
        let mut password = Option::None;
        let mut host = Option::None;
        let mut port = Option::None;
        let mut resource = Option::None;
        if let Some(url) = url {
            if url.contains("://") {
                let url = Url::parse(url)?;
                connect_mode = url.scheme().to_owned();
                username = Some(url.username().to_string());
                password = url.password().map(|val| val.to_string());
                host = url.host_str().map(|val| val.to_string());
                port = url.port();
                resource = Some(url.path().replace("/", "").to_string());

                let query = url.query();

                params = if let Some(param_str) = query {
                    let mut params: HashMap<String, Value> = HashMap::new();
                    let values: Vec<&str> = param_str.split("&").collect();
                    for value in values {
                        let value_str: Vec<&str> = value.split("=").collect();
                        let key = value_str.get(0).ok_or(Error::invalid_type(format!(
                            "failed to get params form {}",
                            url
                        )))?;
                        let value = value_str.get(1);

                        let value = if let Some(val) = value {
                            val.parse::<Value>()?
                        } else {
                            Value::Nil
                        };

                        let _ = params.insert(key.to_string(), value);
                    }

                    params
                } else {
                    HashMap::new()
                };
            } else {
                connect_mode = url.to_string();
            }
        }

        Ok(Self {
            sess_mode: sess_mode.to_owned(),
            ds_mode: ds_mode.to_owned(),
            connect_mode,
            host,
            port,
            resource,
            username,
            password,
            params,
        })
    }

    /// 获取 Session 模式
    #[inline(always)]
    pub fn get_sess_mode(&self) -> &str {
        self.sess_mode.as_str()
    }

    /// 获取 DS 模式
    #[inline(always)]
    pub fn get_ds_mode(&self) -> &str {
        self.ds_mode.as_str()
    }

    /// 获取连接模式
    #[inline(always)]
    pub fn get_connect_mod(&self) -> &str {
        self.connect_mode.as_str()
    }

    /// 获取连接地址
    #[inline(always)]
    pub fn get_host(&self) -> Option<&str> {
        if let Some(host) = &self.host {
            Some(host.as_str())
        } else {
            None
        }
    }

    /// 获取连接端口
    #[inline(always)]
    pub fn get_port(&self) -> Option<u16> {
        self.port
    }

    /// 获取连接资源
    #[inline(always)]
    pub fn get_res(&self) -> Option<String> {
        self.resource.clone()
    }

    /// 获取用户名
    #[inline(always)]
    pub fn get_username(&self) -> Option<String> {
        self.username.clone()
    }

    /// 获取登录密码
    #[inline(always)]
    pub fn get_password(&self) -> Option<&str> {
        if let Some(password) = &self.password {
            Some(password.as_str())
        } else {
            None
        }
    }

    /// 获取指定参数
    pub fn get_param<T: TryFrom<Value, Error = Error>>(&self, name: &str) -> Result<T> {
        self.params
            .get(&name.to_owned())
            .map(|val| T::try_from(val.clone()))
            .ok_or(Error::index_param(name))?
    }
}

impl FromStr for Instance {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Instance::from(s)
    }
}

#[test]
fn test() {
    let instance = Instance::from(
        "sqlite:remote:password://oracle:admin@127.0.0.1:22/bee?connect_timeout=1000",
    )
    .unwrap();

    assert_eq!("sqlite".to_owned(), instance.get_sess_mode());
    assert_eq!("remote".to_owned(), instance.get_ds_mode());
    assert_eq!("password".to_owned(), instance.get_connect_mod());
    assert_eq!(Some("oracle".to_owned()), instance.get_username());
    assert_eq!(Some("admin"), instance.get_password());
    assert_eq!(Some("127.0.0.1"), instance.get_host());
    assert_eq!(Some(22), instance.get_port());
    assert_eq!(Some("bee".to_owned()), instance.get_res());

    let timeout: i32 = instance.get_param("connect_timeout").unwrap();
    assert_eq!(1000_i32, timeout);

    let instance = Instance::from("sqlite:agent:default").unwrap();

    assert_eq!("sqlite".to_owned(), instance.sess_mode);
    assert_eq!("agent".to_owned(), instance.ds_mode);
    assert_eq!("default".to_owned(), instance.connect_mode);
}
