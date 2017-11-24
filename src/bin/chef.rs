extern crate chef;
extern crate chef_api;

extern crate env_logger;
#[macro_use]
extern crate log;

extern crate serde_json;

use chef_api::api_client::*;
use chef::requests::node::*;
use std::fs::File;

pub fn main() {
    env_logger::init().unwrap();
    let client = ApiClient::from_json_config("/Users/thom/.chef/knife.json").unwrap();

    let nodes = client.nodes().get();
    match nodes {
        Ok(nodes) => {
            let nodes: NodeList = nodes.into();
            for n in nodes {
                let node: Node =
                    Node::try_from(client.nodes().name(n.as_ref()).get().unwrap()).unwrap();
                println!("{}", node.name.clone().unwrap());
                info!("{:?}", node);
            }
        }
        Err(e) => println!("{}", e),
    }


    let value = client.nodes().name("data_bag_test").get();
    match value {
        Ok(json) => {
            let node: Node = Node::try_from(json).unwrap();
            info!("{:?}", node);
        }
        Err(e) => println!("{}", e),
    }


    // let output = node.unwrap();
    // info!("{:?}", output);


    let fh = File::open("fixtures/node.json").unwrap();
    // let node2 = Node::from_json(fh).unwrap();
    let node2: Node = serde_json::from_reader(fh).unwrap();
    let value = client.nodes().post(&node2);

    match value {
        Ok(json) => {
            let node: Node = Node::try_from(json).unwrap();
            info!("{:?}", node);
        }
        Err(e) => println!("{}", e),
    }



    // let node2 = Node::from_json(fh).unwrap();
    // info!("Prepare for new node!");
    // info!("{:?}", node2);
    // let out = node2.save(&client);
    // let out = out.unwrap();
    // info!("{:?}", out)


    // let nodes: NodeList = .unwrap().into();
    // // let nodes = NodeList::new(&client);
    // for n in nodes {
    //     println!("{:?}", n)
    // }
}
