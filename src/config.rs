use hyper::Url;
use openssl::crypto::pkey::PKey;
use serde_json;
use serde_json::Value;
use std::path::PathBuf;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::Read;

pub struct Config {
    pub endpoint: Option<Url>,
    pub user: Option<String>,
    pub key: Option<PKey>,
    keypath: Option<String>,
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
            key: None,
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
                fh.read_to_string(&mut data);
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
        self.keypath = Some(keypath.clone());
        match File::open(keypath) {
            Ok(mut fh) => {
                self.key = Some(PKey::private_key_from_pem(&mut fh).unwrap());
                self
            }
            Err(_) => panic!("Couldn't open private key"),
        }
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

    use openssl::crypto::pkey::PKey;
    use std::fs::File;

    const ENDPOINT: &'static str = "https://localhost/organizations/clownco";
    const PRIVATE_KEY: &'static str = "fixtures/spec-user.pem";
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
    fn test_load_key() {
        let mut fh = File::open(PRIVATE_KEY).unwrap();
        let k0 = PKey::private_key_from_pem(&mut fh).unwrap();
        let cfg = Config::new().key(PRIVATE_KEY);
        assert!(cfg.key.unwrap().public_eq(&k0))
    }
}
