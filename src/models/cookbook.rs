model_use!();

chef_json_type!(CookbookVersionJsonClass, "Chef::CookbookVersion");
chef_json_type!(CookbookVersionChefType, "cookbook_version");

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CookbookVersion {
    pub cookbook_name: String,
    chef_type: String,
    pub version: String,
    pub attributes: Vec<HashMap<String, Value>>,
    pub metadata: HashMap<String, Value>,
    pub recipes: Vec<HashMap<String, Value>>,
    providers: Vec<HashMap<String, Value>>,
    resources: Vec<HashMap<String, Value>>,
    templates: Vec<HashMap<String, Value>>,
    root_files: Vec<HashMap<String, Value>>,
    #[serde(rename = "frozen?")]
    pub frozen: bool,
    json_class: String,
}

model_impl!(CookbookVersion);
model_list!(CookbookVersionList);
