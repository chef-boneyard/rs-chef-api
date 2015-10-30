use hyper::Url;
use openssl::crypto::pkey::PKey;
use std::path::PathBuf;
use std::env;
use std::fs::File;

pub struct Config {
    endpoint: Option<Url>,
    user: Option<String>,
    key: Option<PKey>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            endpoint: None,
            user: None,
            key: None,
        }
    }

    // /// Load a configuration file from JSON
    // pub fn from_json(path: &str) -> Config {
    // }

    pub fn key(mut self, path: &str) -> Config {
        let keypath = get_absolute_path(path);
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
