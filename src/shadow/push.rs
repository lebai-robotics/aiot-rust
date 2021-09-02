use crate::alink::{SysAck, ALINK_VERSION, global_id_next};
use crate::shadow::base::*;
use serde_json::Value;
use serde::{Serialize,Deserialize};

// 影子设备更新
// /shadow/update/${YourProductKey}/${YourDeviceName}
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShadowUpdateRequest {
	method: String,
	state: Value,
	version: u32,
}

impl crate::shadow::Runner {
	// 影子设备更新
	pub async fn update(&self, value: Value, version: u32) -> crate::Result<()> {
		let payload = ShadowUpdateRequest {
			method: "update".to_string(),
			state: value,
			version,
		};
		self.publish(format!("/shadow/update/{}/{}", self.three.product_key, self.three.device_name), &payload).await
	}
}