use api_client::{ApiClient, Error};
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use std::io;
use std::io::{Cursor, Read, ErrorKind};

chef_json_type!(NodeJsonClass, "Chef::Node");
chef_json_type!(NodeChefType, "node");

#[derive(Debug,Clone,Serialize,Deserialize,Default)]
pub struct Node {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    chef_type: NodeChefType,
    #[serde(default)]
    json_class: NodeJsonClass,
    #[serde(default)]
    chef_environment: String,
    #[serde(default)]
    run_list: Vec<String>,
    #[serde(default)]
    normal: HashMap<String, Value>,
    #[serde(default)]
    automatic: HashMap<String, Value>,
    #[serde(default)]
    default: HashMap<String, Value>,
    #[serde(default,rename(json="override"))]
    overrides: HashMap<String, Value>,
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
