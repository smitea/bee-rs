use crate::{Error, Value};
use std::{collections::HashMap, convert::TryFrom, str::FromStr};
use url::Url;

#[derive(Debug, Clone)]
pub struct Instance {
    connect_mode: String,
    host: Option<String>,
    port: Option<u16>,
    resource: String,
    username: String,
    password: Option<String>,
    params: HashMap<String, Value>,
}

impl Instance {
    pub fn from(url: &str) -> Result<Self, Error> {
        let url = Url::parse(url)?;
        let connect_mode = url.scheme();
        let username = url.username().to_string();
        let password = url.password().map(|val| val.to_string());
        let host = url.host_str().map(|val| val.to_string());
        let port = url.port();
        let resource = url.path().replace("/", "").to_string();

        let query = url.query();

        let params: HashMap<String, Value> = if let Some(param_str) = query {
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

                params.insert(key.to_string(), value);
            }

            params
        } else {
            HashMap::new()
        };

        Ok(Self {
            connect_mode: connect_mode.to_string(),
            host,
            port,
            resource,
            username,
            password,
            params,
        })
    }

    #[inline(always)]
    pub fn get_connect_mod(&self) -> &str {
        self.connect_mode.as_str()
    }

    #[inline(always)]
    pub fn get_host(&self) -> Option<&str> {
        if let Some(host) = &self.host {
            Some(host.as_str())
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn get_port(&self) -> Option<u16> {
        self.port
    }

    #[inline(always)]
    pub fn get_res(&self) -> &str {
        self.resource.as_str()
    }

    #[inline(always)]
    pub fn get_username(&self) -> &str {
        self.username.as_str()
    }

    #[inline(always)]
    pub fn get_password(&self) -> Option<&str> {
        if let Some(password) = &self.password {
            Some(password.as_str())
        } else {
            None
        }
    }

    pub fn get_param<T: TryFrom<Value, Error = Error>>(&self, name: &str) -> Result<T, Error> {
        self.params
            .get(&name.to_owned())
            .map(|val| T::try_from(val.clone()))
            .ok_or(Error::index_param(name))?
    }
}

impl FromStr for Instance {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Instance::from(s)
    }
}

#[test]
fn test() {
    let instance =
        Instance::from("ssh://oracle:admin@127.0.0.1:22/bee?connect_timeout=1000").unwrap();

    assert_eq!("ssh".to_owned(), instance.connect_mode);
    assert_eq!("oracle".to_owned(), instance.username);
    assert_eq!(Some("admin".to_owned()), instance.password);
    assert_eq!(Some("127.0.0.1".to_owned()), instance.host);
    assert_eq!(Some(22), instance.port);
    assert_eq!("bee".to_owned(), instance.resource);

    let timeout: i32 = instance.get_param("connect_timeout").unwrap();
    assert_eq!(1000_i32, timeout);
}

#[test]
#[should_panic(expected = "relative URL without a base")]
fn test_faild1() {
    let _ = Instance::from("://oracle:admin@127.0.0.1:22/bee?connect_timeout=1000").unwrap();
}

#[test]
#[should_panic(expected = "relative URL without a base")]
fn test_faild2() {
    let _ = Instance::from("//ssh:oracle:admin@127.0.0.1:?connect_timeout=1000").unwrap();
}
