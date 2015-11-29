use api_client::{ApiClient, Error};
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use std::io;
use std::io::{Cursor, Read, ErrorKind};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Node {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    chef_type: String,
    #[serde(default)]
    json_class: String,
    #[serde(default)]
    chef_environment: String,
    #[serde(default)]
    run_list: Option<Vec<String>>,
    #[serde(default)]
    normal: Option<HashMap<String, Value>>,
    #[serde(default)]
    automatic: Option<HashMap<String, Value>>,
    #[serde(default)]
    default: Option<HashMap<String, Value>>,
    #[serde(default,rename(json="override"))]
    overrides: Option<HashMap<String, Value>>,
}

impl Default for Node {
    fn default() -> Node {
        Node {
            name: None,
            chef_type: String::from("node"),
            json_class: String::from("Chef::Node"),
            chef_environment: String::from("_default"),
            run_list: Some(vec![]),
            normal: Some(HashMap::new()),
            automatic: Some(HashMap::new()),
            default: Some(HashMap::new()),
            overrides: Some(HashMap::new()),
        }
    }
}

impl Read for Node {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Ok(node) = serde_json::to_vec(self) {
            let mut node = Cursor::new(node.as_ref() as &[u8]);
            Read::read(&mut node, buf)
        } else {
            Err(io::Error::new(ErrorKind::InvalidData, "Failed to convert node to JSON"))
        }
    }
}

impl Node {
    pub fn fetch(client: &ApiClient, name: &str) -> Result<Node, Error> {
        let org = &client.config.organization_path();
        let path = format!("{}/nodes/{}", org, name);
        client.get(path.as_ref()).and_then(|r| r.from_json::<Node>())
    }

    pub fn save(&self, client: &ApiClient) -> Result<Node, Error> {
        let name = &self.name.clone().unwrap();
        let org = &client.config.organization_path();
        let path = format!("{}/nodes/{}", org, name);
        client.put(path.as_ref(), self).and_then(|r| r.from_json::<Node>())
    }

    pub fn from_json<R>(r: R) -> Result<Node, Error>
        where R: Read
    {
        serde_json::from_reader::<R, Node>(r).map_err(|e| Error::Json(e))
    }
}

#[cfg(test)]
mod tests {
    use super::Node;

    #[test]
    fn test_node_defaults() {
        let n = Node::default();
        assert_eq!(n.json_class, "Chef::Node")
    }
}
