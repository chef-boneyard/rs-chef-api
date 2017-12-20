# A Chef Client API library

This library implements the raw requests and authentication methods
necessary to interact with a [Chef] Server.

See the [Chef Server API] documentation for further information on the possible requests.

## Connecting

You'll need a credentials file as documented in [RFC 99].

To retrieve a list of cookbook names, first create an `ApiClient` and then make a
request to the cookbook endpoint:

```rust,no_run
use chef_api::api_client::{ApiClient, Execute};

let client = ApiClient::from_credentials(None).unwrap();
let cookbooks = client.cookbooks().get();
```

This crate uses [`serde`] to serialize requests from JSON, meaning
everything will return a `serde_json::Value`.

The chef crate provides models that are easier to work with.

[Chef]: https://www.chef.io/chef/
[Chef Server API]: https://chef-server-api-docs.chef.io/
[RFC 99]: https://chef.github.io/chef-rfc/rfc099-authentication-config-file.html
[`serde`]: https://serde.rs/

