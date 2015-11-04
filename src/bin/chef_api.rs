extern crate chef_api;

#[macro_use]
extern crate log;
extern crate env_logger;

use chef_api::api_client::ApiClient;
use chef_api::config::Config;
use chef_api::requests::node::NodesRequest;

use std::io::Read;

pub fn main() {
    env_logger::init().unwrap();
    let cfg = Config::from_json("/Users/thom/.chef/knife.json");
    let client = ApiClient::new(cfg);
    let nodes = NodesRequest::new();

    let res = client.run(nodes);

    let mut output = String::new();
    res.unwrap().read_to_string(&mut output);
    info!("{:?}", output)
}
