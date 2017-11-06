use api_client::ApiClient;
use utils::decode_list;
use serde_json::value::Value;
use std::collections::hash_map::HashMap;
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
    // Return Cookbook Metadata
    pub fn show(client: &ApiClient, name: String) -> Result<CookbookMetadata> {
        let org = &client.config.organization_path();
        let path = format!("{}/cookbooks/{}/_latest", org, name);
        client.get::<CookbookMetadata>(path.as_ref())
    }
    pub fn version(client: &ApiClient, name: String, version: String) -> Result<CookbookMetadata> {
        let org = &client.config.organization_path();
        let path = format!("{}/cookbooks/{}/{}", org, name, version);
        client.get::<CookbookMetadata>(path.as_ref())
    }
}

#[derive(Debug)]
pub struct CookbooksList {
    count: usize,
    pub cookbooks: Vec<String>,
    client: ApiClient,
}


impl CookbooksList {
    pub fn new(client: &ApiClient) -> Self {
        let org = &client.config.organization_path();
        let path = format!("{}/cookbooks", org);
        client
            .get(path.as_ref())
            .and_then(decode_list)
            .and_then(|list| {
                Ok(CookbooksList {
                    cookbooks: list,
                    count: 0,
                    client: client.clone(),
                })
            })
            .unwrap()
    }
}

// Itenarator for Cookbooks
impl Iterator for CookbooksList {
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
