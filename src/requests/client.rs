use api_client::ApiClient;
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use std::io;
use std::io::{Cursor, Read};
use std::io::ErrorKind as IoErrorKind;
use utils::decode_list;
use errors::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Client {
    #[serde(default)]
    pub name: String,
    clientname: String,
    validator: bool,
    orgname: String,
    json_class: String,
    chef_type: String,
}

#[derive(Debug)]
pub struct Clients {
    count: usize,
    name: Vec<String>,
    client: ApiClient,
}

impl Clients {
    pub fn list(client: &ApiClient) -> Clients {
        let org = &client.config.organization_path();
        let path = format!("{}/clients", org);
        client
            .get(path.as_ref())
            .and_then(decode_list)
            .and_then(|list| {
                Ok(Clients {
                    count: 0,
                    name: list,
                    client: client.clone(),
                })
            })
            .unwrap()
    }
    pub fn show(client: &ApiClient, name: String) -> Result<Client> {
        let org = &client.config.organization_path();
        let path = format!("{}/clients/{}", org, name);
        client.get(path.as_ref()).and_then(
            |r| r.from_json::<Client>(),
        )
    }

    pub fn delete(client: &ApiClient, name: String) -> Result<Client> {
        let org = &client.config.organization_path();
        let path = format!("{}/clients/{}", org, name);
        client.delete(path.as_ref()).and_then(
            |r| r.from_json::<Client>(),
        )
    }
}

// Itenarator for Clients
impl Iterator for Clients {
    type Item = Result<Client>;

    fn count(self) -> usize {
        self.name.len()
    }

    fn next(&mut self) -> Option<Result<Client>> {
        if self.name.len() >= 1 {
            Some(Clients::show(&self.client, self.name.remove(0)))
        } else {
            None
        }
    }
}
