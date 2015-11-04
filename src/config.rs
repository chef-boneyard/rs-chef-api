use hyper::Url;
use serde_json;
use serde_json::Value;
use std::path::PathBuf;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::Read;

#[derive(Clone)]
pub struct Config {
    pub endpoint: Option<Url>,
    pub user: Option<String>,
    pub keypath: Option<String>,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Config")
            .field("endpoint", &self.endpoint)
            .field("user", &self.user)
            .field("keypath", &self.keypath)
            .finish()
    }
}

impl Config {
    pub fn new() -> Config {
        Config {
            endpoint: None,
            user: None,
            keypath: None,
        }
    }

    /// Load a configuration file from JSON
    pub fn from_json(path: &str) -> Config {
        let path = get_absolute_path(path);
        let cfg = Config::new();
        match File::open(path) {
            Ok(mut fh) => {
                let mut data = String::new();
                let _ = fh.read_to_string(&mut data);
                let val: Value = serde_json::from_str(data.as_ref()).unwrap();
                let obj = val.as_object().unwrap();
                let cfg = cfg.key(obj.get("client_key").unwrap().as_string().unwrap());
                let cfg = cfg.endpoint(obj.get("chef_server_url").unwrap().as_string().unwrap());
                let cfg = cfg.user(obj.get("node_name").unwrap().as_string().unwrap());
                cfg
            },
            Err(_) => panic!("Couldn't open config file")
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

    pub fn user(mut self, user: &str) -> Config {
        self.user = Some(String::from(user));
        self
    }

    pub fn organization_path(&self) -> String {
        match self.endpoint {
            Some(ref endpoint) => {
                endpoint.serialize_path().unwrap()
            },
            None => panic!("Can't find an endpoint")
        }
    }

    pub fn url_base(&self) -> String {
        match self.endpoint {
            Some(ref endpoint) => {
                let host = &endpoint.serialize_host().unwrap();
                let port = &endpoint.port_or_default().unwrap();
                let scheme = &endpoint.scheme;
                format!("{}://{}:{}", scheme, host, port)
            },
            None => panic!("Can't find an endpoint")
        }
    }

}

fn get_absolute_path(val: &str) -> String {
    let mut p = PathBuf::from(val);
    if !p.is_absolute() {
        p = env::current_dir().unwrap();
        p.push(val);
    }
    p.to_str().unwrap().to_string()
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
        assert_eq!(cfg.organization_path(),
                   "/organizations/clownco")
    }

    #[test]
    fn test_config_base_url() {
        let cfg = Config::new().endpoint(ENDPOINT);
        assert_eq!(cfg.url_base(), "https://localhost:443")
    }

}
