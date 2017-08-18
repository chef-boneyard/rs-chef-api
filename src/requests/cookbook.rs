use api_client::ApiClient;
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use std::io;
use std::io::{Cursor, Read};
use std::io::ErrorKind as IoErrorKind;
use utils::decode_list;
use errors::*;

// Struct for Cookbook List from /_latest URL
#[derive(Debug)]
pub struct Cookbooks {
    count: usize,
    cookbooks: Vec<String>,
    client: ApiClient,
}

// Struct for Cookbook Metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CookbookMetadata {
    pub cookbook_name: String,
    chef_type: String,
    pub version: String,
    pub attributes: Vec<HashMap<String, Value>>,
    pub metadata: HashMap<String, Value>,
    pub recipes: Vec<HashMap<String, Value>>,
    providers: Vec<HashMap<String, Value>>,
    resources: Vec<HashMap<String, Value>>,
    templates: Vec<HashMap<String, Value>>,
    root_files: Vec<HashMap<String, Value>>,
    #[serde(default, rename = "frozen?")]
    pub frozen: bool,
    json_class: String,
}

//
impl Cookbooks {
    // Return Cookbook Array from Last Version
    pub fn new(client: &ApiClient) -> Cookbooks {
        let org = &client.config.organization_path();
        let path = format!("{}/cookbooks/_latest", org);
        client
            .get(path.as_ref())
            .and_then(decode_list)
            .and_then(|list| {
                Ok(Cookbooks {
                    count: 0,
                    cookbooks: list,
                    client: client.clone(),
                })
            })
            .unwrap()
    }
    // Retrun Cookbook Metadata
    pub fn show(client: &ApiClient, name: String) -> Result<CookbookMetadata> {
        let org = &client.config.organization_path();
        let path = format!("{}/cookbooks/{}/_latest", org, name);
        client.get(path.as_ref()).and_then(|r| {
            r.from_json::<CookbookMetadata>()
        })
    }
}

// Itenarator for Cookbooks
impl Iterator for Cookbooks {
    type Item = Result<CookbookMetadata>;

    fn count(self) -> usize {
        self.cookbooks.len()
    }

    fn next(&mut self) -> Option<Result<CookbookMetadata>> {
        if self.cookbooks.len() >= 1 {
            Some(Cookbooks::show(&self.client, self.cookbooks.remove(0)))
        } else {
            None
        }
    }
}
