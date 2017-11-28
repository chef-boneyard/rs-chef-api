use chef_api::utils::decode_list;
use failure::Error;

use serde_json::Value;
use std::collections::HashMap;

use std::io;
use serde_json;
use std::io::{Cursor, Read};
use std::io::ErrorKind as IoErrorKind;

chef_json_type!(NodeJsonClass, "Chef::Node");
chef_json_type!(NodeChefType, "node");

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Node {
    #[serde(default)] pub name: Option<String>,
    #[serde(default)] chef_type: NodeChefType,
    #[serde(default)] json_class: NodeJsonClass,
    #[serde(default)] pub chef_environment: String,
    #[serde(default)] pub run_list: Vec<String>,
    #[serde(default)] pub normal: HashMap<String, Value>,
    #[serde(default)] pub automatic: HashMap<String, Value>,
    #[serde(default)] pub default: HashMap<String, Value>,
    #[serde(default, rename = "override")] pub overrides: HashMap<String, Value>,
}

impl Read for Node {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Ok(node) = serde_json::to_vec(self) {
            let mut node = Cursor::new(node.as_ref() as &[u8]);
            Read::read(&mut node, buf)
        } else {
            Err(io::Error::new(
                IoErrorKind::InvalidData,
                "Failed to convert node to JSON",
            ))
        }
    }
}

impl Node {
    pub fn try_from(val: Value) -> Result<Self, Error> {
        serde_json::from_value(val).map_err(|e| e.into())
    }
}
// impl Node {
//     pub fn new<S>(name: S) -> Self
//     where
//         S: Into<String>,
//     {
//         Node {
//             name: Some(name.into()),
//             ..Default::default()
//         }
//     }

//     pub fn fetch<S: Into<String>>(client: &ApiClient, name: S) -> Result<Node> {
//         let org = &client.config.organization_path();
//         let path = format!("{}/nodes/{}", org, name.into());
//         client.get::<Node>(path.as_ref())
//     }

//     pub fn save(&self, client: &ApiClient) -> Result<Node> {
//         let name = &self.name.clone().unwrap();
//         let org = &client.config.organization_path();
//         let path = format!("{}/nodes/{}", org, name);
//         client.put::<&Node, Node>(path.as_ref(), &self)
//     }

//     pub fn delete(&self, client: &ApiClient) -> Result<Node> {
//         let name = &self.name.clone().unwrap();
//         let org = &client.config.organization_path();
//         let path = format!("{}/nodes/{}", org, name);
//         client.delete::<Node>(path.as_ref())
//     }
// }

// pub fn delete_node(client: &ApiClient, name: &str) -> Result<Node> {
//     let org = &client.config.organization_path();
//     let path = format!("{}/nodes/{}", org, name);
//     client.delete::<Node>(path.as_ref())
// }

#[derive(Debug)]
pub struct NodeList {
    count: usize,
    nodes: Vec<String>,
}

impl From<Value> for NodeList {
    fn from(list: Value) -> Self {
        decode_list(&list)
            .and_then(|list| {
                Ok(NodeList {
                    nodes: list,
                    count: 0,
                })
            })
            .unwrap()
    }
}

impl Iterator for NodeList {
    type Item = String;

    fn count(self) -> usize {
        self.nodes.len()
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.nodes.len() >= 1 {
            Some(self.nodes.remove(0))
        // Some(Node::fetch(&self.client, self.nodes.remove(0)))
        } else {
            None
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::Node;
//     use std::fs::File;

//     #[test]
//     fn test_node_from_file() {
//         let fh = File::open("fixtures/node.json").unwrap();
//         let node = Node::from_json(fh).unwrap();
//         assert_eq!(node.name.unwrap(), "test")
//     }
// }
