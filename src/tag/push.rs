use crate::alink::{global_id_next, AlinkRequest, SysAck, ALINK_VERSION};
use crate::tag::base::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// 标签信息上报
// /sys/{productKey}/{deviceName}/thing/deviceinfo/update
pub type DeviceInfoUpdateRequest = AlinkRequest<Vec<DeviceInfoKeyValue>>;

// 标签信息删除
// /sys/{productKey}/{deviceName}/thing/deviceinfo/delete
pub type DeviceInfoDeleteRequest = AlinkRequest<Vec<DeviceInfoKey>>;

impl crate::tag::Runner {
	// 标签信息上报
	pub async fn update(&self, infos: Vec<DeviceInfoKeyValue>, ack: bool) -> crate::Result<()> {
		let payload = DeviceInfoUpdateRequest {
			id: global_id_next().to_string(),
			version: ALINK_VERSION.to_string(),
			params: infos,
			sys: Some(SysAck { ack: ack.into() }),
			method: Some("thing.deviceinfo.update".to_string()),
		};
		self
			.publish(
				format!(
					"/sys/{}/{}/thing/deviceinfo/update",
					self.three.product_key, self.three.device_name
				),
				&payload,
			)
			.await
	}

	// 标签信息删除
	pub async fn delete(&self, keys: &[&str], ack: bool) -> crate::Result<()> {
		let payload = DeviceInfoDeleteRequest {
			id: global_id_next().to_string(),
			version: ALINK_VERSION.to_string(),
			params: keys.iter().map(|n| DeviceInfoKey {
				attr_key: String::from(*n),
			}).collect(),
			sys: Some(SysAck { ack: ack.into() }),
			method: Some("thing.deviceinfo.delete".to_string()),
		};
		self
			.publish(
				format!(
					"/sys/{}/{}/thing/deviceinfo/delete",
					self.three.product_key, self.three.device_name
				),
				&payload,
			)
			.await
	}
}
