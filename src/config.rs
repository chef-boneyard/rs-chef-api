use hyper::Url;
use serde_json;
use serde_json::Value;
use std::path::PathBuf;
use std::env;
use std::fs::File;

#[derive(Clone,Debug)]
pub struct Config {
    pub endpoint: Option<Url>,
    pub user: Option<String>,
    pub keypath: Option<String>,
    pub sign_ver: String,
}

impl Config {
    pub fn new() -> Config {
        Config {
            endpoint: None,
            user: None,
            keypath: None,
            sign_ver: String::from("1.1"),
        }
    }

    /// Load a configuration file from JSON
    pub fn from_json(path: &str) -> Config {
        let path = get_absolute_path(path);
        let cfg = Config::new();
        match File::open(path) {
            Ok(fh) => {
                let val: Value = serde_json::from_reader(fh).unwrap();
                let obj = val.as_object().unwrap();
                let cfg = cfg.key(obj.get("client_key").unwrap().as_string().unwrap());
                let cfg = cfg.endpoint(obj.get("chef_server_url").unwrap().as_string().unwrap());
                let cfg = cfg.user(obj.get("node_name").unwrap().as_string().unwrap());
                cfg.sign_ver(obj.get("sign_ver").unwrap().as_string().unwrap())
            }
            Err(_) => panic!("Couldn't open config file"),
        }
    }

    pub fn key(mut self, path: &str) -> Config {
        let keypath = get_absolute_path(path);
        self.keypath = Some(keypath);
        self
    }

    pub fn endpoint(mut self, endpoint: &str) -> Config {
        let url = match Url::parse(endpoint) {
            Ok(url) => url,
            Err(_) => panic!("Please provide a valid URL"),
        };
        self.endpoint = Some(url);
        self
    }

    pub fn user<S>(mut self, user: S) -> Config
        where S: Into<String>
    {
        self.user = Some(user.into());
        self
    }

    pub fn sign_ver<S>(mut self, sign_ver: S) -> Config
        where S: Into<String>
    {
        self.sign_ver = sign_ver.into();
        self
    }

    pub fn organization_path(&self) -> String {
        match self.endpoint {
            Some(ref endpoint) => endpoint.serialize_path().unwrap(),
            None => panic!("Can't find an endpoint"),
        }
    }

    pub fn url_base(&self) -> String {
        match self.endpoint {
            Some(ref endpoint) => {
                let host = &endpoint.serialize_host().unwrap();
                let port = &endpoint.port_or_default().unwrap();
                let scheme = &endpoint.scheme;
                format!("{}://{}:{}", scheme, host, port)
            }
            None => panic!("Can't find an endpoint"),
        }
    }
}

fn get_absolute_path(val: &str) -> String {
    let mut p = PathBuf::from(val);
    if !p.is_absolute() {
        p = env::current_dir().unwrap();
        p.push(val);
    }
    p.to_str().unwrap().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENDPOINT: &'static str = "https://localhost/organizations/clownco";
    const CONFIG: &'static str = "fixtures/config.json";

    #[test]
    fn test_from_json() {
        let cfg = Config::from_json(CONFIG);
        assert_eq!(cfg.endpoint.unwrap().serialize_path().unwrap(),
                   "/organizations/rs_chef_api")
    }

    #[test]
    fn test_config_endpoint() {
        let cfg = Config::new().endpoint(ENDPOINT);
        assert_eq!(cfg.endpoint.unwrap().serialize_path().unwrap(),
                   "/organizations/clownco")
    }

    #[test]
    fn test_config_organization() {
        let cfg = Config::new().endpoint(ENDPOINT);
        assert_eq!(cfg.organization_path(), "/organizations/clownco")
    }

    #[test]
    fn test_config_base_url() {
        let cfg = Config::new().endpoint(ENDPOINT);
        assert_eq!(cfg.url_base(), "https://localhost:443")
    }

    #[test]
    fn test_default_sign_ver() {
        let cfg = Config::new();
        assert_eq!(cfg.sign_ver, "1.1")
    }
}
