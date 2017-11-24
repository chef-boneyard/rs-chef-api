use api_client::ApiClient;
use serde_json::Value;
use serde::Serialize;
use errors::*;

use std::rc::Rc;

pub struct NodeQuery {
    pub(crate) client: Rc<ApiClient>,
    pub(crate) path: String,
}

impl NodeQuery {
    pub fn name(&self, name: &str) -> Self {
        let path = add_path_element(self.path, name);
        Self {
            self.client,
            path,
        }
    }
}
