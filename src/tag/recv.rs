use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::any::TypeId;
use crate::Error;
use log::*;
use crate::alink_topic::ALinkSubscribeTopic;
use spin::Lazy;
use crate::alink::AlinkResponse;

// 标签信息上报响应
// /sys/{productKey}/{deviceName}/thing/deviceinfo/update_reply
pub type DeviceInfoUpdateResponse = AlinkResponse<()>;

// 标签信息删除响应
// /sys/{productKey}/{deviceName}/thing/deviceinfo/delete_replly
pub type DeviceInfoDeleteResponse = AlinkResponse<()>;

pub static TOPICS: Lazy<Vec<ALinkSubscribeTopic>> = Lazy::new(|| vec![
	ALinkSubscribeTopic::new("sys/+/+/thing/deviceinfo/update_reply", TypeId::of::<DeviceInfoUpdateResponse>()),
	ALinkSubscribeTopic::new("sys/+/+/thing/deviceinfo/delete_replly", TypeId::of::<DeviceInfoDeleteResponse>()),
]);

pub enum TagRecv {
	DeviceInfoUpdateResponse(DeviceInfoUpdateResponse),
	DeviceInfoDeleteResponse(DeviceInfoDeleteResponse),
}

#[async_trait::async_trait]
impl crate::Executor for crate::tag::Executor {
	async fn execute(&self, topic: &str, payload: &[u8]) -> crate::Result<()> {
		debug!("receive: {} {}", topic, String::from_utf8_lossy(payload));
		for item in &*TOPICS {
			if !item.is_match(topic, &self.three.product_key, &self.three.device_name) {
				return Ok(());
			}
			let data = match item.payload_type_id {
				a if a == TypeId::of::<DeviceInfoUpdateResponse>() => {
					TagRecv::DeviceInfoUpdateResponse(serde_json::from_slice(&payload)?)
				}
				a if a == TypeId::of::<DeviceInfoDeleteResponse>() => {
					TagRecv::DeviceInfoDeleteResponse(serde_json::from_slice(&payload)?)
				}
				_ => {
					return Ok(());
				}
			};
			self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
		}
		Ok(())
	}
}