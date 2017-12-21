model_use!();

chef_json_type!(DataBagItemJsonClass, "Chef::DataBagItem");
chef_json_type!(DataBagItemChefType, "data_bag_item");

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DataBagItem {
    id: Option<String>,
    data_bag: Option<String>,
    chef_type: DataBagItemChefType,
    json_class: DataBagItemJsonClass,
    pub raw_data: HashMap<String, Value>,
}

model_impl!(DataBagItem);
model_list!(DataBagItemList);
