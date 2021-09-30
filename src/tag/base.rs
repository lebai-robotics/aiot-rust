use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfoKeyValue {
    pub attr_key: String,
    pub attr_value: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfoKey {
    pub attr_key: String,
}
