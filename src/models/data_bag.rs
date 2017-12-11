model_use!();

chef_json_type!(DataBagJsonClass, "Chef::DataBag");
chef_json_type!(DataBagChefType, "data_bag");

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DataBag {
    pub name: Option<String>,
    chef_type: DataBagChefType,
    json_class: DataBagJsonClass,
    id: Option<usize>,
}

model_impl!(DataBag);
model_list!(DataBagList);
