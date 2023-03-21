model_use!();

chef_json_type!(ClientJsonClass, "Chef::Client");
chef_json_type!(ClientChefType, "client");

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Client {
    #[serde(default)]
    pub name: String,
    clientname: String,
    validator: bool,
    orgname: String,
    json_class: ClientJsonClass,
    chef_type: ClientChefType,
}

model_impl!(Client);
model_list!(ClientList);
model_result!(Client, ClientResult);
