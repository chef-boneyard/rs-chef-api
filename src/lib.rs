#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]

// #![deny(missing_docs,
//         missing_debug_implementations, missing_copy_implementations,
//         trivial_casts, trivial_numeric_casts,
//         unsafe_code,
//         unstable_features,
//         unused_import_braces, unused_qualifications)]

extern crate openssl;
extern crate rustc_serialize;
extern crate chrono;

#[macro_use]
extern crate hyper;

#[macro_use]
extern crate log;

extern crate serde;
extern crate serde_json;

pub mod authentication;
mod http_headers;
mod utils;
pub mod config;
pub mod api_client;
pub mod requests;
