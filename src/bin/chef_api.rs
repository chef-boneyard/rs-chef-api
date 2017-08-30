extern crate chef_api;

extern crate env_logger;
#[macro_use]
extern crate log;

use chef_api::api_client::ApiClient;
use chef_api::requests::node::*;
// use std::fs::File;

pub fn main() {
    env_logger::init().unwrap();
    let client = ApiClient::from_json_config("/Users/thom/.chef/knife.json").unwrap();
    let node = Node::fetch(&client, "data_bag_test");

    let output = node.unwrap();
    info!("{:?}", output);


    // let fh = File::open("fixtures/node.json").unwrap();
    // let node2 = Node::from_json(fh).unwrap();
    // info!("Prepare for new node!");
    // info!("{:?}", node2);
    // let out = node2.save(&client);
    // let out = out.unwrap();
    // info!("{:?}", out)


    let nodes = NodeList::new(&client);
    for n in nodes {
        println!("{:?}", n.unwrap().name)
    }
}
