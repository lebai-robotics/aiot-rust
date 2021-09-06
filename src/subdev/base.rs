use crate::util::auth::{SIGN_METHOD, sign_device};
use std::time::{UNIX_EPOCH, SystemTime};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfoId {
	pub device_name: String,
	pub product_key: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
	pub device_name: String,
	pub product_key: String,
	pub sign: String,
	pub sign_method: String,
	pub timestamp: String,
	pub client_id: String,
	// 如果取值是true，则清理所有子设备离线时的消息，即所有未接收的QoS1消息将被清除。如果取值是false，则不清理子设备离线时的消息。
	pub clean_session: Option<String>,

}

impl DeviceInfo {
	pub fn new(product_key: String, device_name: String, clean_session: Option<bool>, device_secret: String) -> Self {
		// client_id+device_name+product_key+timestamp;
		let client_id = format!("{}&{}", product_key, device_name);
		let start = SystemTime::now();
		let since_the_epoch = start.duration_since(UNIX_EPOCH)
			.expect("Time went backwards");
		let timestamp = since_the_epoch.as_millis();
		let sign = sign_device(&client_id, &device_name, &product_key, &device_secret, timestamp);
		Self {
			device_name,
			product_key,
			sign,
			sign_method: String::from(SIGN_METHOD),
			timestamp: timestamp.to_string(),
			client_id,
			clean_session: clean_session.map(|n| String::from(if n { "true" } else { "false" })),
		}
	}
}