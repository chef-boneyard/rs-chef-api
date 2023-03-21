#[macro_export]
macro_rules! chef_json_type {
    ($id:ident, $val:expr) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct $id(String);
        impl Default for $id {
            fn default() -> $id {
                $id(String::from($val))
            }
        }
    };
}

macro_rules! model_list {
    ($id:ident) => {
        #[derive(Debug)]
        pub struct $id {
            items: Vec<String>,
        }

        impl From<Value> for $id {
            fn from(list: Value) -> Self {
                decode_list(&list)
                    .and_then(|list| Ok(Self { items: list }))
                    .unwrap()
            }
        }

        impl Iterator for $id {
            type Item = String;

            fn count(self) -> usize {
                self.items.len()
            }

            fn next(&mut self) -> Option<Self::Item> {
                if self.items.len() >= 1 {
                    Some(self.items.remove(0))
                } else {
                    None
                }
            }
        }
    };
}

macro_rules! model_result {
    ($model:ident, $id:ident) => {
        #[derive(Debug)]
        pub struct $id {
            items: Vec<$model>,
        }

        impl From<Value> for $id {
            fn from(list: Value) -> Self {
                assert!(list.is_object());
                let list = list.get("rows").unwrap();

                assert!(list.is_array());
                let mut output: Vec<$model> = Vec::new();

                for json_node in list.as_array().unwrap().to_owned() {
                    output.push($model::try_from(json_node).unwrap());
                }

                $id { items: output }
            }
        }

        impl Iterator for $id {
            type Item = $model;

            fn count(self) -> usize {
                self.items.len()
            }

            fn next(&mut self) -> Option<Self::Item> {
                if self.items.len() >= 1 {
                    Some(self.items.remove(0))
                } else {
                    None
                }
            }
        }
    };
}

macro_rules! model_impl {
    ($id:ident) => {
        impl Read for $id {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                if let Ok(item) = serde_json::to_vec(self) {
                    let mut item = Cursor::new(item.as_ref() as &[u8]);
                    Read::read(&mut item, buf)
                } else {
                    Err(io::Error::new(
                        IoErrorKind::InvalidData,
                        "Failed to convert to JSON",
                    ))
                }
            }
        }

        impl $id {
            pub fn try_from(val: Value) -> Result<Self, Error> {
                serde_json::from_value(val).map_err(|e| e.into())
            }
        }
    };
}

macro_rules! model_use {
    () => {
        use chef_api::utils::decode_list;
        use failure::Error;

        use serde_json;
        use serde_json::Value;

        #[allow(unused_imports)]
        use std::collections::HashMap;
        use std::io;
        use std::io::ErrorKind as IoErrorKind;
        use std::io::{Cursor, Read};
    };
}

pub mod node;
pub use self::node::*;
pub mod environment;
pub use self::environment::*;
pub mod role;
pub use self::role::*;
pub mod data_bag;
pub use self::data_bag::*;
pub mod data_bag_item;
pub use self::data_bag_item::*;
pub mod cookbook;
pub use self::cookbook::*;
pub mod client;
pub use self::client::*;
