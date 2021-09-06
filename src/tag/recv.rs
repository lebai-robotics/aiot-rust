use crate::alink::AlinkResponse;
use crate::{alink_topic::ALinkSubscribeTopic, Error};
use log::*;
use std::any::TypeId;

use enum_iterator::IntoEnumIterator;
use enum_kinds::EnumKind;
use serde::{Deserialize, Serialize};
// 标签信息上报响应
// /sys/{productKey}/{deviceName}/thing/deviceinfo/update_reply
pub type DeviceInfoUpdateResponse = AlinkResponse;

// 标签信息删除响应
// /sys/{productKey}/{deviceName}/thing/deviceinfo/delete_replly
pub type DeviceInfoDeleteResponse = AlinkResponse;

#[derive(Debug, EnumKind)]
#[enum_kind(TagRecvKind, derive(Serialize, IntoEnumIterator, Deserialize))]
pub enum TagRecv {
	DeviceInfoUpdateResponse(DeviceInfoUpdateResponse),
	DeviceInfoDeleteResponse(DeviceInfoDeleteResponse),
}

impl TagRecvKind {
	pub fn match_kind(topic: &str, product_key: &str, device_name: &str) -> Option<TagRecvKind> {
		for item in TagRecvKind::into_enum_iter() {
			let alink_topic = item.get_topic();
			if !alink_topic.is_match(topic, product_key, device_name) {
				continue;
			}
			return Some(item);
			// self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
		}
		None
	}
	pub fn to_payload(&self, payload: &[u8]) -> crate::Result<TagRecv> {
		match *self {
			TagRecvKind::DeviceInfoUpdateResponse => Ok(TagRecv::DeviceInfoUpdateResponse(
				serde_json::from_slice(&payload)?,
			)),
			TagRecvKind::DeviceInfoDeleteResponse => Ok(TagRecv::DeviceInfoDeleteResponse(
				serde_json::from_slice(&payload)?,
			)),
		}
	}
	
	pub fn get_topic(&self) -> ALinkSubscribeTopic {
		match *self {
			TagRecvKind::DeviceInfoUpdateResponse => {
				ALinkSubscribeTopic::new("/sys/{}/{}/thing/deviceinfo/update_reply")
			}
			TagRecvKind::DeviceInfoDeleteResponse => {
				ALinkSubscribeTopic::new("/sys/{}/{}/thing/deviceinfo/delete_replly")
			}
		}
	}
}

#[async_trait::async_trait]
impl crate::Executor for crate::tag::Executor {
	async fn execute(&self, topic: &str, payload: &[u8]) -> crate::Result<()> {
		debug!("receive: {} {}", topic, String::from_utf8_lossy(payload));
		if let Some(kind) = TagRecvKind::match_kind(topic, &self.three.product_key, &self.three.device_name){
			let data = kind.to_payload(payload)?;
			self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
		}
		debug!("no match topic: {}", topic);
		Ok(())
	}
}
