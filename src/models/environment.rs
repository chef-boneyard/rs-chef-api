model_use!();
chef_json_type!(EnvironmentJsonClass, "Chef::Environment");
chef_json_type!(EnvironmentChefType, "environment");

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Environment {
    pub name: Option<String>,
    pub description: Option<String>,
    chef_type: EnvironmentChefType,
    json_class: EnvironmentJsonClass,
    pub cookbook_versions: HashMap<String, String>,
    pub default_attributes: HashMap<String, Value>,
    pub override_attributes: HashMap<String, Value>,
}

model_impl!(Environment);
model_list!(EnvironmentList);
