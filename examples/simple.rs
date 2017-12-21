extern crate chef;
extern crate chef_api;

extern crate env_logger;
#[macro_use]
extern crate log;

extern crate serde_json;

use chef_api::api_client::*;
use chef::models::*;

pub fn main() {
    env_logger::init().unwrap();
    let client = ApiClient::from_credentials(None).unwrap();

    let nodes = client.nodes().get();
    info!("{:?}", nodes);
    match nodes {
        Ok(nodes) => {
            let nodes: NodeList = nodes.into();
            for n in nodes {
                let node: Node = Node::try_from(client.nodes().node(&n).get().unwrap()).unwrap();
                println!("{}", node.name.clone().unwrap());
                info!("{:?}", node);
            }
        }
        Err(e) => println!("{}", e),
    }

    let value = client.nodes().node("data_bag_test").get();
    match value {
        Ok(json) => {
            let node: Node = Node::try_from(json).unwrap();
            info!("{:?}", node);
        }
        Err(e) => println!("{}", e),
    }

    let value = client.users().get();
    info!("{:?}", value);

    let value = client
        .cookbooks()
        .cookbook("windows")
        .version("3.0.5")
        .get();
    info!("{:?}", value);

    let universe = client.universe().get();
    info!("{:?}", universe);

    // let fh = File::open("fixtures/node.json").unwrap();
    // You'd not actually need the line below, but it gives you a chance to ensure the json file is
    // valid - otherwise just post `fh`.
    // let node2: Node = serde_json::from_reader(fh).unwrap();
    // let value = client.nodes().post(&node2);

    // match value {
    //     Ok(json) => {
    //         let node: Node = Node::try_from(json).unwrap();
    //         info!("{:?}", node);
    //     }
    //     Err(e) => println!("{}", e),
    // }
}
