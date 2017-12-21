//! A Chef Client API library
//!
//! This library implements the raw requests and authentication methods
//! necessary to interact with a [Chef] Server.
//!
//! See the [Chef Server API] documentation for further information on the possible requests.
//!
//! ## Connecting
//!
//! You'll need a credentials file as documented in [RFC
//! 99].
//!
//! To retrieve a list of cookbook names, first create an `ApiClient` and then make a
//! request to the cookbook endpoint:
//!
//! ```rust,no_run
//! use chef_api::api_client::{ApiClient, Execute};
//!
//! let client = ApiClient::from_credentials(None).unwrap();
//! let cookbooks = client.cookbooks().get();
//! ```
//!
//! This crate uses [`serde`] to serialize requests from JSON.
//!
//! [Chef]: https://www.chef.io/chef/
//! [Chef Server API]: https://chef-server-api-docs.chef.io/
//! [RFC 99]: https://chef.github.io/chef-rfc/rfc099-authentication-config-file.html
//! [`serde`]: https://serde.rs/
//!
#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]
// #![deny(missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts,
//         trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
//         unused_qualifications)]

#[macro_use]
extern crate failure;

extern crate chrono;
extern crate openssl;
extern crate rustc_serialize;
extern crate url;

extern crate futures;

#[macro_use]
extern crate hyper;
extern crate hyper_openssl;
extern crate tokio_core;

#[macro_use]
extern crate log;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate toml;

pub use errors::*;
pub mod errors;
pub mod authentication;
mod http_headers;
#[macro_use]
mod macros;
pub mod utils;
pub mod credentials;

pub mod api_client;

pub use requests::*;
pub mod requests;
