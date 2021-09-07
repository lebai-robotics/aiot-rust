use crate::util::auth::{SIGN_METHOD, sign, sign_device};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfoId {
	pub device_name: String,
	pub product_key: String,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfoWithSecret {
	pub device_name: String,
	pub product_key: String,
	pub device_secret: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
	pub device_name: String,
	pub product_key: String,
	pub sign: String,
	pub sign_method: String,
	pub timestamp: String,
	pub client_id: Option<String>,
	// 如果取值是true，则清理所有子设备离线时的消息，即所有未接收的QoS1消息将被清除。如果取值是false，则不清理子设备离线时的消息。
	pub clean_session: Option<String>,
}

impl DeviceInfo {
	pub fn new(
		product_key: String,
		device_name: String,
		clean_session: Option<bool>,
		device_secret: String,
		client_id: Option<String>,
	) -> Self {
		// client_id+device_name+product_key+timestamp;
		let client_id = client_id.unwrap_or(format!("{}&{}", product_key, device_name));
		let start = SystemTime::now();
		let since_the_epoch = start
			.duration_since(UNIX_EPOCH)
			.expect("Time went backwards");
		let timestamp = since_the_epoch.as_millis();

		let res = format!(
			"clientId{}deviceName{}productKey{}timestamp{}",
			client_id,
			device_name,
			product_key,
			timestamp.to_string()
		);
		let sign = sign(&res, &device_secret);
		Self {
			device_name,
			product_key,
			sign,
			sign_method: String::from(SIGN_METHOD),
			timestamp: timestamp.to_string(),
			client_id: Some(client_id),
			clean_session: clean_session.map(|n| String::from(if n { "true" } else { "false" })),
		}
	}
}
