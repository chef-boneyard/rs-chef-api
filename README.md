# Chef objects

This library depends upon `chef_api` for most functionality, but
provides a set of models for common Chef objects, such as nodes, roles
and environments.

# Usage

Models implement a version of`try_from()` in the context of [`serde_json`]'s Value
type - which is what is returned by all requests.

```rust
use chef_api::api_client::{ApiClient, Execute};
use chef::models::Node;

let client = ApiClient::from_credentials(None)?;
let node = client.nodes().node("my_node").get()?;
let node: Node = Node::try_from(node)?;
println!("Node name is {}", node.name.unwrap());
```

Once `try_from` is stablised in Rust, we'll switch to that.

## Lists 

Many APIs in the Chef Server return a list of items. Models will try to
convert those lists in to `Iterator`s:

```rust
use chef_api::api_client::{ApiClient, Execute};
use chef::models::NodeList;

let client = ApiClient::from_credentials(None)?;
let nodes: NodeList = client.nodes().get()?.into();
for n in nodes { 
  println!("saw node: {}", n);
}
```

