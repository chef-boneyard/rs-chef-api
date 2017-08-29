use api_client::ApiClient;
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use std::io;
use std::io::{Cursor, Read};
use std::io::ErrorKind as IoErrorKind;
use utils::decode_list;
use errors::*;

chef_json_type!(EnvironmentJsonClass, "Chef::Environment");
chef_json_type!(EnvironmentChefType, "environment");

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Environment {
    #[serde(default)] pub name: Option<String>,
    #[serde(default)] pub description: Option<String>,
    #[serde(default)] chef_type: EnvironmentChefType,
    #[serde(default)] json_class: EnvironmentJsonClass,
    #[serde(default)] pub cookbook_versions: HashMap<String, String>,
    #[serde(default)] pub default_attributes: HashMap<String, Value>,
    #[serde(default, rename = "override")] pub override_attributes: HashMap<String, Value>,
}

impl Read for Environment {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Ok(environment) = serde_json::to_vec(self) {
            let mut environment = Cursor::new(environment.as_ref() as &[u8]);
            Read::read(&mut environment, buf)
        } else {
            Err(io::Error::new(
                IoErrorKind::InvalidData,
                "Failed to convert environment to JSON",
            ))
        }
    }
}

impl Environment {
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Environment {
            name: Some(name.into()),
            ..Default::default()
        }
    }

    pub fn fetch<S: Into<String>>(client: &ApiClient, name: S) -> Result<Environment> {
        let org = &client.config.organization_path();
        let path = format!("{}/environments/{}", org, name.into());
        client.get::<Environment>(path.as_ref())
    }

    pub fn save(&self, client: &ApiClient) -> Result<Environment> {
        let name = &self.name.clone().unwrap();
        let org = &client.config.organization_path();
        let path = format!("{}/environments/{}", org, name);
        client.put::<&Environment, Environment>(path.as_ref(), &self)
    }

    pub fn delete(&self, client: &ApiClient) -> Result<Environment> {
        let name = &self.name.clone().unwrap();
        let org = &client.config.organization_path();
        let path = format!("{}/environments/{}", org, name);
        client.delete::<Environment>(path.as_ref())
    }

    pub fn from_json<R>(r: R) -> Result<Environment>
    where
        R: Read,
    {
        Ok(try!(serde_json::from_reader::<R, Environment>(r)))
    }
}

pub fn delete_environment(client: &ApiClient, name: &str) -> Result<Environment> {
    let org = &client.config.organization_path();
    let path = format!("{}/environments/{}", org, name);
    client.delete::<Environment>(path.as_ref())
}

#[derive(Debug)]
pub struct EnvironmentList {
    count: usize,
    environments: Vec<String>,
    client: ApiClient,
}

impl EnvironmentList {
    pub fn new(client: &ApiClient) -> Self {
        let org = &client.config.organization_path();
        let path = format!("{}/environments", org);
        client
            .get(path.as_ref())
            .and_then(decode_list)
            .and_then(|list| {
                Ok(EnvironmentList {
                    environments: list,
                    count: 0,
                    client: client.clone(),
                })
            })
            .unwrap()
    }
}

impl Iterator for EnvironmentList {
    type Item = Result<Environment>;

    fn count(self) -> usize {
        self.environments.len()
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.environments.len() >= 1 {
            Some(Environment::fetch(
                &self.client,
                self.environments.remove(0),
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Environment;
    use std::fs::File;

    #[test]
    fn test_environment_from_file() {
        let fh = File::open("fixtures/environment.json").unwrap();
        let environment = Environment::from_json(fh).unwrap();
        assert_eq!(environment.name.unwrap(), "test")
    }
}
