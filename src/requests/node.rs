use api_client::{ApiClient, Error};
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use std::io;
use std::io::{Cursor, Read, ErrorKind};
use utils::decode_list;

chef_json_type!(NodeJsonClass, "Chef::Node");
chef_json_type!(NodeChefType, "node");

#[derive(Debug,Clone,Serialize,Deserialize,Default)]
pub struct Node {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    chef_type: NodeChefType,
    #[serde(default)]
    json_class: NodeJsonClass,
    #[serde(default)]
    pub chef_environment: String,
    #[serde(default)]
    pub run_list: Vec<String>,
    #[serde(default)]
    pub normal: HashMap<String, Value>,
    #[serde(default)]
    pub automatic: HashMap<String, Value>,
    #[serde(default)]
    pub default: HashMap<String, Value>,
    #[serde(default,rename(json="override"))]
    pub overrides: HashMap<String, Value>,
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
    pub fn new(name: &str) -> Node {
        Node { name: Some(String::from(name)), ..Default::default() }
    }

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

    pub fn delete(&self, client: &ApiClient) -> Result<Node, Error> {
        let name = &self.name.clone().unwrap();
        let org = &client.config.organization_path();
        let path = format!("{}/nodes/{}", org, name);
        client.delete(path.as_ref()).and_then(|r| r.from_json::<Node>())
    }

    pub fn from_json<R>(r: R) -> Result<Node, Error>
        where R: Read
    {
        serde_json::from_reader::<R, Node>(r).map_err(|e| Error::Json(e))
    }
}

pub fn delete_node(client: &ApiClient, name: &str) -> Result<Node, Error> {
    let org = &client.config.organization_path();
    let path = format!("{}/nodes/{}", org, name);
    client.delete(path.as_ref()).and_then(|r| r.from_json::<Node>())
}

#[derive(Debug)]
pub struct NodeList {
    count: usize,
    nodes: Vec<String>,
    client: ApiClient,
}

impl NodeList {
    pub fn new(client: &ApiClient) -> NodeList {
        let org = &client.config.organization_path();
        let path = format!("{}/nodes", org);
        client.get(path.as_ref())
              .and_then(|r| decode_list(r))
              .and_then(|list| {
                  Ok(NodeList {
                      nodes: list,
                      count: 0,
                      client: client.clone(),
                  })
              })
              .unwrap()
    }
}

impl Iterator for NodeList {
    type Item = Result<Node, Error>;

    fn count(self) -> usize {
        self.nodes.len()
    }

    fn next(&mut self) -> Option<Result<Node, Error>> {
        if self.count < self.nodes.len() {
            let ref name = self.nodes[self.count];
            self.count += 1;
            Some(Node::fetch(&self.client, name.as_ref()))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Node;
    use std::fs::File;

    #[test]
    fn test_node_from_file() {
        let fh = File::open("fixtures/node.json").unwrap();
        let node = Node::from_json(fh).unwrap();
        assert_eq!(node.name.unwrap(), "test")
    }
}
