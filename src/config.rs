use url::Url;
use serde_json;
use serde_json::Value;
use std::path::PathBuf;
use std::env;
use std::fs::File;
use errors::*;

#[derive(Clone,Debug,Default)]
pub struct Config {
    pub endpoint: Option<Url>,
    pub user: Option<String>,
    pub keypath: Option<String>,
    pub sign_ver: String,
}

impl Config {
    pub fn new() -> Config {
        Config { sign_ver: String::from("1.3"), ..Default::default() }
    }

    /// Load a configuration file from JSON
    pub fn from_json(path: &str) -> Result<Config> {
        let path = get_absolute_path(path);
        let fh = try!(File::open(&path));
        let val: Value = try!(serde_json::from_reader(fh));

        if let Some(ref obj) = val.as_object() {
            let key = obj.get("client_key").unwrap();
            let key: String = try!(serde_json::from_value(key.clone()));
            let key = get_absolute_path(key.as_ref());

            let endpoint = obj.get("chef_server_url").unwrap();
            let endpoint: String = try!(serde_json::from_value(endpoint.clone()));

            let user = obj.get("node_name").unwrap();
            let user: String = try!(serde_json::from_value(user.clone()));

            let sign_ver: String = "1.3".into();

            let endpoint = try!(Url::parse(endpoint.as_ref()));

            Ok(Config {
                endpoint: Some(endpoint),
                user: Some(user),
                keypath: Some(key),
                sign_ver: sign_ver,
            })
        } else {
            Err(ErrorKind::UnparseableConfigError(path).into())
        }
    }

    pub fn organization_path(&self) -> String {
        match self.endpoint {
            Some(ref endpoint) => endpoint.path().into(),
            None => panic!("Can't find an endpoint"),
        }
    }

    pub fn url_base(&self) -> String {
        match self.endpoint {
            Some(ref endpoint) => {
                let host = &endpoint.host_str().unwrap();
                let port = &endpoint.port_or_known_default().unwrap();
                let scheme = &endpoint.scheme();
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
    use url::Url;

    const ENDPOINT: &'static str = "https://localhost/organizations/clownco";
    const CONFIG: &'static str = "fixtures/config.json";

    #[test]
    fn test_from_json() {
        let cfg = Config::from_json(CONFIG).unwrap();
        assert_eq!(cfg.endpoint.unwrap().path(), "/organizations/rs_chef_api")
    }

    #[test]
    fn test_config_endpoint() {
        let url = Url::parse(ENDPOINT).unwrap();
        let cfg = Config {
            endpoint: Some(url),
            user: None,
            keypath: None,
            sign_ver: String::from("1.3"),
        };
        assert_eq!(cfg.endpoint.unwrap().path(), "/organizations/clownco")
    }

    #[test]
    fn test_config_organization() {
        let url = Url::parse(ENDPOINT).unwrap();
        let cfg = Config {
            endpoint: Some(url),
            user: None,
            keypath: None,
            sign_ver: String::from("1.3"),
        };
        assert_eq!(cfg.organization_path(), "/organizations/clownco")
    }

    #[test]
    fn test_config_base_url() {
        let url = Url::parse(ENDPOINT).unwrap();
        let cfg = Config {
            endpoint: Some(url),
            user: None,
            keypath: None,
            sign_ver: String::from("1.3"),
        };
        assert_eq!(cfg.url_base(), "https://localhost:443")
    }

    #[test]
    fn test_default_sign_ver() {
        let cfg = Config::new();
        assert_eq!(cfg.sign_ver, "1.3")
    }
}
