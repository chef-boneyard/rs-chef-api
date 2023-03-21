extern crate chef;
extern crate chef_api;

extern crate env_logger;
#[macro_use]
extern crate log;

extern crate serde_json;

use chef::models::*;
use chef_api::api_client::*;

pub fn main() {
    env_logger::init().unwrap();
    let client = ApiClient::from_credentials(None).unwrap();

    println!("Starting search");
    let n = client
        .search()
        .search_index("node")
        .q("role:rb_vault_server")
        .get()
        .unwrap();
    println!("Done searching");

    let nr: NodeResult = n.into();
    for n in nr {
        println!("{}", n.name.clone().unwrap());
        // info!("{:?}", n);
    }
}
