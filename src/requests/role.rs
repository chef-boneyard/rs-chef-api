use api_client::{ApiClient, Error};
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use std::io;
use std::io::{Cursor, Read, ErrorKind};
use utils::decode_list;

chef_json_type!(RoleJsonClass, "Chef::Role");
chef_json_type!(RoleChefType, "role");

#[derive(Debug,Clone,Serialize,Deserialize,Default)]
pub struct Role {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    chef_type: RoleChefType,
    #[serde(default)]
    json_class: RoleJsonClass,
    #[serde(default)]
    pub run_list: Vec<String>,
    #[serde(default)]
    pub default_attributes: HashMap<String, Value>,
    #[serde(default)]
    pub override_attributes: HashMap<String, Value>,
}

impl Read for Role {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Ok(role) = serde_json::to_vec(self) {
            let mut role = Cursor::new(role.as_ref() as &[u8]);
            Read::read(&mut role, buf)
        } else {
            Err(io::Error::new(ErrorKind::InvalidData, "Failed to convert role to JSON"))
        }
    }
}

impl Role {
    pub fn new<S>(name: S) -> Role
        where S: Into<String>
    {
        Role { name: Some(name.into()), ..Default::default() }
    }

    pub fn fetch<S: Into<String>>(client: &ApiClient, name: S) -> Result<Role, Error> {
        let org = &client.config.organization_path();
        let path = format!("{}/roles/{}", org, name.into());
        client.get(path.as_ref()).and_then(|r| r.from_json::<Role>())
    }

    pub fn save(&self, client: &ApiClient) -> Result<Role, Error> {
        let name = &self.name.clone().unwrap();
        let org = &client.config.organization_path();
        let path = format!("{}/roles/{}", org, name);
        client.put(path.as_ref(), self).and_then(|r| r.from_json::<Role>())
    }

    pub fn delete(&self, client: &ApiClient) -> Result<Role, Error> {
        let name = &self.name.clone().unwrap();
        let org = &client.config.organization_path();
        let path = format!("{}/roles/{}", org, name);
        client.delete(path.as_ref()).and_then(|r| r.from_json::<Role>())
    }

    pub fn from_json<R>(r: R) -> Result<Role, Error>
        where R: Read
    {
        serde_json::from_reader::<R, Role>(r).map_err(|e| Error::Json(e))
    }
}

pub fn delete_role(client: &ApiClient, name: &str) -> Result<Role, Error> {
    let org = &client.config.organization_path();
    let path = format!("{}/roles/{}", org, name);
    client.delete(path.as_ref()).and_then(|r| r.from_json::<Role>())
}

#[derive(Debug)]
pub struct RoleList {
    count: usize,
    roles: Vec<String>,
    client: ApiClient,
}

impl RoleList {
    pub fn new(client: &ApiClient) -> RoleList {
        let org = &client.config.organization_path();
        let path = format!("{}/roles", org);
        client.get(path.as_ref())
              .and_then(|r| decode_list(r))
              .and_then(|list| {
                  Ok(RoleList {
                      roles: list,
                      count: 0,
                      client: client.clone(),
                  })
              })
              .unwrap()
    }
}

impl Iterator for RoleList {
    type Item = Result<Role, Error>;

    fn count(self) -> usize {
        self.roles.len()
    }

    fn next(&mut self) -> Option<Result<Role, Error>> {
        if self.count < self.roles.len() {
            let ref name = self.roles[self.count];
            self.count += 1;
            Some(Role::fetch(&self.client, name.as_ref()))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Role;
    use std::fs::File;

    #[test]
    fn test_role_from_file() {
        let fh = File::open("fixtures/role.json").unwrap();
        let role = Role::from_json(fh).unwrap();
        assert_eq!(role.name.unwrap(), "test")
    }
}
