extern crate chef_api;

#[macro_use]
extern crate log;
extern crate env_logger;

use chef_api::api_client::ApiClient;
use chef_api::requests::node::NodesRequest;

pub fn main() {
    env_logger::init().unwrap();
    let client = ApiClient::from_json_config("/Users/thom/.chef/knife.json");
    let nodes = NodesRequest::new();

    let res = client.run(nodes);

    let output = res.unwrap().body;
    info!("{:?}", output)
}
