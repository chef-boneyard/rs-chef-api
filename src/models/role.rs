model_use!();

chef_json_type!(RoleJsonClass, "Chef::Role");
chef_json_type!(RoleChefType, "role");

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Role {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    chef_type: RoleChefType,
    #[serde(default)]
    json_class: RoleJsonClass,
    #[serde(default)]
    pub run_list: Vec<String>,
    #[serde(default)]
    pub default_attributes: HashMap<String, Value>,
    #[serde(default)]
    pub override_attributes: HashMap<String, Value>,
}

model_impl!(Role);
model_list!(RoleList);
model_result!(Role, RoleResult);
