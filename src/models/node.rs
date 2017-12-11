model_use!();

chef_json_type!(NodeJsonClass, "Chef::Node");
chef_json_type!(NodeChefType, "node");

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Node {
    pub name: Option<String>,
    chef_type: NodeChefType,
    json_class: NodeJsonClass,
    pub chef_environment: String,
    pub run_list: Vec<String>,
    pub normal: HashMap<String, Value>,
    pub automatic: HashMap<String, Value>,
    pub default: HashMap<String, Value>,
    #[serde(rename = "override")] pub overrides: HashMap<String, Value>,
}

model_impl!(Node);
model_list!(NodeList);
