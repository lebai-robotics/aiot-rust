use crate::alink::alink_topic::ALinkSubscribeTopic;
use crate::alink::AlinkRequest;
use crate::{alink::AlinkResponse, Error};
use enum_iterator::IntoEnumIterator;
use enum_kinds::EnumKind;
use log::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use spin::Lazy;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapNotifyParams {
	/// 目前唯一取值为0，表示设备发生分发，期望设备重新请求Bootstrap接入点。
	pub cmd: u32,
}

pub type BootstrapNotify = AlinkRequest<BootstrapNotifyParams>;


#[derive(Debug, EnumKind)]
#[enum_kind(BootstrapRecvKind, derive(Serialize, IntoEnumIterator, Deserialize))]
pub enum BootstrapRecv {
	/// 设备分发通知
	BootstrapNotify(BootstrapNotify),
}

impl BootstrapRecvKind {
	pub fn match_kind(
		topic: &str,
		product_key: &str,
		device_name: &str,
	) -> Option<BootstrapRecvKind> {
		for item in BootstrapRecvKind::into_enum_iter() {
			let alink_topic = item.get_topic();
			if !alink_topic.is_match(topic, product_key, device_name) {
				continue;
			}
			return Some(item);
			// self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
		}
		None
	}
	pub fn to_payload(&self, payload: &[u8]) -> crate::Result<BootstrapRecv> {
		let json_str = String::from_utf8_lossy(&payload).replace(",\"data\":{},", ",\"data\":null,");
		match *self {
			BootstrapRecvKind::BootstrapNotify => Ok(BootstrapRecv::BootstrapNotify(
				serde_json::from_str(&json_str)?,
			)),
		}
	}
	pub fn get_topic(&self) -> ALinkSubscribeTopic {
		match *self {
			BootstrapRecvKind::BootstrapNotify => {
				ALinkSubscribeTopic::new_we("/sys/+/+/thing/bootstrap/notify")
			}
		}
	}
}

#[async_trait::async_trait]
impl crate::Executor for crate::bootstrap::Executor<BootstrapRecv> {
	async fn execute(&self, topic: &str, payload: &[u8]) -> crate::Result<()> {
		debug!("receive: {} {}", topic, String::from_utf8_lossy(payload));
		if let Some(kind) =
			BootstrapRecvKind::match_kind(topic, &self.three.product_key, &self.three.device_name)
		{
			let data = kind.to_payload(payload)?;
			self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
		} else {
			debug!("no match topic: {}", topic);
		}
		Ok(())
	}
}
