#[macro_export]
macro_rules! chef_json_type {
    // A simple type that allows us to use serde's Default implementation.
    // FIXME: revisit when https://github.com/serde-rs/serde/issues/90 gets fixed
    ($id:ident, $val:expr) => {

        #[derive(Debug,Clone,Serialize,Deserialize)]
        struct $id(String);
        impl Default for $id {
            fn default() -> $id {
                $id(String::from($val))
            }
        }

    }
}

pub mod node;
pub mod environment;
pub mod role;
pub mod data_bag;
pub mod data_bag_item;

