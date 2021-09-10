use serde_json::Value;
use crate::Error;
use log::*;
use crate::alink::alink_topic::ALinkSubscribeTopic;
use spin::Lazy;
use serde::{Serialize,Deserialize};
use enum_iterator::IntoEnumIterator;
use enum_kinds::EnumKind;


#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShadowGetTopic {
	pub method: String,
	pub payload: Value,
	pub timestamp: u64,
}

#[derive(Debug, EnumKind)]
#[enum_kind(ShadowRecvKind, derive(Serialize, IntoEnumIterator, Deserialize))]
pub enum ShadowRecv {
	/// 设备主动获取影子内容响应
	ShadowGetTopic(ShadowGetTopic),
}
impl ShadowRecvKind {
	pub fn match_kind(topic: &str, product_key: &str, device_name: &str) -> Option<ShadowRecvKind> {
		for item in ShadowRecvKind::into_enum_iter() {
			let alink_topic = item.get_topic();
			if !alink_topic.is_match(topic, product_key, device_name) {
				continue;
			}
			return Some(item);
			// self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
		}
		None
	}
	pub fn to_payload(&self, payload: &[u8]) -> crate::Result<ShadowRecv> {
		let json_str = String::from_utf8_lossy(&payload).replace(",\"data\":{},", ",\"data\":null,");
		match *self {
			ShadowRecvKind::ShadowGetTopic => Ok(ShadowRecv::ShadowGetTopic(
				serde_json::from_str(&json_str)?,
			)),
		}
	}
	
	pub fn get_topic(&self) -> ALinkSubscribeTopic {
		match *self {
			ShadowRecvKind::ShadowGetTopic => {
				ALinkSubscribeTopic::new_we("/shadow/get/+/+")
			}
		}
	}
}


#[async_trait::async_trait]
impl crate::Executor for crate::shadow::Executor {
	async fn execute(&self, topic: &str, payload: &[u8]) -> crate::Result<()> {
		debug!("receive: {} {}", topic, String::from_utf8_lossy(payload));
		if let Some(kind) = ShadowRecvKind::match_kind(topic, &self.three.product_key, &self.three.device_name){
			let data = kind.to_payload(payload)?;
			self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
		} else {
			debug!("no match topic: {}", topic);
		}
		Ok(())
	}
}