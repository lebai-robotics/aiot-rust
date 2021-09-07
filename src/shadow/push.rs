use crate::alink::{global_id_next, SysAck, ALINK_VERSION};
use crate::shadow::base::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// 影子设备更新
// /shadow/update/${YourProductKey}/${YourDeviceName}
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShadowUpdateRequest {
	method: String,
	state: Option<Value>,
	version: Option<u32>,
}

impl crate::shadow::Runner {
	// 影子设备属性更新
	pub async fn update(&self, value: Value, version: u32) -> crate::Result<()> {
		let payload = ShadowUpdateRequest {
			method: "update".to_string(),
			state: Some(value),
			version: Some(version),
		};
		self
			.publish(
				format!(
					"/shadow/update/{}/{}",
					self.three.product_key, self.three.device_name
				),
				&payload,
			)
			.await
	}
	// 影子设备属性获取
	pub async fn get(&self) -> crate::Result<()> {
		let payload = ShadowUpdateRequest {
			method: "get".to_string(),
			state: None,
			version: None,
		};
		self
			.publish(
				format!(
					"/shadow/update/{}/{}",
					self.three.product_key, self.three.device_name
				),
				&payload,
			)
			.await
	}
	// 影子设备属性删除
	pub async fn delete(&self, value: Value, version: u32) -> crate::Result<()> {
		let payload = ShadowUpdateRequest {
			method: "delete".to_string(),
			state: Some(value),
			version: Some(version),
		};
		self
			.publish(
				format!(
					"/shadow/update/{}/{}",
					self.three.product_key, self.three.device_name
				),
				&payload,
			)
			.await
	}
}
