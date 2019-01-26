use dirs;
use errors::*;
use failure::Error;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use toml::Value;
use url::Url;

/// Representation of a Chef configuration.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Config {
    /// The URL to the Chef Server organization
    pub chef_server_url: String,
    node_name: Option<String>,
    client_name: Option<String>,
    #[serde(skip)]
    profile: String,
    client_key: String,
    /// The path or contents of the validator key
    pub validator_key: Option<String>,
    /// The authentication scheme to use; defaults to 1.3.
    #[serde(default = "default_auth_string")]
    pub sign_ver: String,
}

impl Config {
    /// Creates a new Config from a `TOML` string.
    pub fn from_str(toml: &str, profile: &str) -> Result<Self, Error> {
        let credentials = toml.parse::<Value>().unwrap();
        let credentials = credentials[profile].clone();
        let mut creds: Config = credentials
            .try_into()
            .map_err(ChefError::TomlDeserializeError)?;
        creds.profile = profile.into();
        Ok(creds)
    }

    /// Loads a config from `~/.chef/credentials`, using the following heuristic to determine a
    /// profile name:
    ///
    /// - If `profile` is not `None`, use the value provided.
    /// - Using the `CHEF_PROFILE` environment variable
    /// - Using the contents of `~/.chef/context`
    /// - Otherwise use the default profile.
    pub fn from_credentials(profile: Option<&str>) -> Result<Self, Error> {
        let credentials = get_chef_path("credentials")?;
        let profile = select_profile_name(profile);
        debug!(
            "Opening credentials file: {:?} with profile: {:?}",
            credentials, profile
        );
        match File::open(&credentials) {
            Ok(mut fh) => {
                let mut config = String::new();
                fh.read_to_string(&mut config)?;
                Self::from_str(&config, &profile)
            }
            Err(_) => Err(ChefError::UnparseableConfigError(String::from(
                "Unable to read credentials file",
            ))
            .into()),
        }
    }

    /// Returns the configured name to authenticate with. A profile may use either `client_name` or
    /// `node_name` interchangeably; `client_name` is preferred, and a `DuplicateClientNameError` will
    /// be returned if both `client_name` and `node_name` are used.
    pub fn client_name(&self) -> Result<&str, Error> {
        let profile = self.profile.clone();
        if self.client_name.is_some() && self.node_name.is_some() {
            Err(ChefError::DuplicateClientNameError(profile).into())
        } else if self.node_name.is_some() {
            self.node_name
                .as_ref()
                .ok_or_else(|| {
                    ChefError::UnparseableConfigError(format!(
                        "failed to read node name for profile: {}",
                        profile
                    ))
                    .into()
                })
                .and_then(|n| Ok(n.as_ref()))
        } else if self.client_name.is_some() {
            self.client_name
                .as_ref()
                .ok_or_else(|| {
                    ChefError::UnparseableConfigError(format!(
                        "failed to read client name for profile: {}",
                        profile
                    ))
                    .into()
                })
                .and_then(|n| Ok(n.as_ref()))
        } else {
            Err(ChefError::UnparseableConfigError(format!(
                "No node_name or client_name found for profile: {}",
                profile
            ))
            .into())
        }
    }

    /// Returns the contents of the client key used for signing requests.
    pub fn key(&self) -> Result<Vec<u8>, Error> {
        if self
            .client_key
            .starts_with("-----BEGIN RSA PRIVATE KEY-----")
        {
            Ok(self.client_key.as_bytes().into())
        } else {
            let key_path = get_chef_path(&self.client_key)?;
            debug!("Reading key from {:?}", key_path);
            let mut key: Vec<u8> = vec![];
            File::open(&key_path).and_then(|mut fh| fh.read_to_end(&mut key))?;
            debug!("Read private key: {:?}", key);
            Ok(key)
        }
    }

    fn endpoint(&self) -> Result<Url, Error> {
        Url::parse(self.chef_server_url.as_ref()).map_err(|e| e.into())
    }

    /// Returns the organization path of the configured `chef_server_url`.
    pub fn organization_path(&self) -> Result<String, Error> {
        let endpoint = self.endpoint()?;
        Ok(endpoint.path().into())
    }

    /// Returns the scheme, host and port of the configured `chef_server_url`.
    pub fn url_base(&self) -> Result<String, Error> {
        let endpoint = self.endpoint()?;
        let host = &endpoint.host_str().unwrap();
        let port = &endpoint.port_or_known_default().unwrap();
        let scheme = &endpoint.scheme();
        Ok(format!("{}://{}:{}", scheme, host, port))
    }
}

fn select_profile_name(name: Option<&str>) -> String {
    if name.is_some() {
        debug!("Using explicit profile name");
        return name.unwrap().into();
    };
    if let Ok(env) = env::var("CHEF_PROFILE") {
        debug!("Using environment variable for profile name");
        return env;
    };

    let path = match get_chef_path("context") {
        Ok(p) => p,
        Err(_) => {
            debug!("Unable to figure out path to context file, using default profile");
            return String::from("default");
        }
    };

    if let Ok(mut fh) = File::open(path) {
        debug!("Reading profile context from file");
        let mut contents = String::new();
        return match fh.read_to_string(&mut contents) {
            Ok(_) => contents,
            Err(_) => {
                info!("Failed to read profile from file, using default");
                String::from("default")
            }
        };
    }
    debug!("Using default profile name");
    String::from("default")
}

fn get_chef_path(val: &str) -> Result<String, Error> {
    let home_dir = match dirs::home_dir() {
        Some(path) => path,
        None => {
            return Err(ChefError::PrivateKeyError(String::from(
                "Could not identify user's home directory",
            ))
            .into());
        }
    };
    let mut p = PathBuf::from(val);
    if !p.is_absolute() {
        p = home_dir;
        p.push(".chef");
        p.push(val)
    }
    match p.to_str() {
        Some(path) => Ok(path.to_owned()),
        None => Err(ChefError::PrivateKeyError(String::from(
            "Could not construct a path to the user's .chef directory",
        ))
        .into()),
    }
}

fn default_auth_string() -> String {
    "1.3".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std;

    const CREDENTIALS: &'static str = r#"
    [default]
    node_name = 'barney'
    client_key = 'barney_rubble.pem'
    chef_server_url = 'https://api.chef.io/organizations/bedrock'
    "#;

    #[test]
    fn test_from_str() {
        let cfg = Config::from_str(CREDENTIALS, "default").unwrap();
        assert_eq!(cfg.node_name.unwrap(), "barney")
    }

    #[test]
    fn test_organization_path() {
        let cfg = Config::from_str(CREDENTIALS, "default").unwrap();
        assert_eq!(cfg.organization_path().unwrap(), "/organizations/bedrock")
    }

    #[test]
    fn test_default_profile_name() {
        std::env::remove_var("CHEF_PROFILE");
        assert_eq!(select_profile_name(None), String::from("default"))
    }

    #[test]
    fn test_environment_profile_name() {
        std::env::set_var("CHEF_PROFILE", "environment");
        assert_eq!(select_profile_name(None), String::from("environment"));
        std::env::remove_var("CHEF_PROFILE");
    }

    #[test]
    fn test_explicit_profile_name() {
        assert_eq!(
            select_profile_name(Some("explicit")),
            String::from("explicit")
        )
    }

    #[test]
    fn test_get_chef_path() {
        let home = dirs::home_dir().unwrap();
        std::env::set_var("HOME", "/home/barney");
        let path = get_chef_path("credentials").unwrap();
        assert_eq!(path, "/home/barney/.chef/credentials");
        std::env::set_var("HOME", home);
    }

    #[test]
    fn test_absolute_get_chef_path() {
        let home = dirs::home_dir().unwrap();
        std::env::set_var("HOME", "/home/barney");
        let path = get_chef_path("/home/fred/.chef/fred.pem").unwrap();
        assert_eq!(path, "/home/fred/.chef/fred.pem");
        std::env::set_var("HOME", home);
    }
}
