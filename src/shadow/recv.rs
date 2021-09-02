use serde_json::Value;
use std::collections::HashMap;
use std::any::TypeId;
use crate::Error;
use log::*;
use crate::alink_topic::ALinkSubscribeTopic;
use spin::Lazy;
use crate::shadow::base::*;
use serde::{Serialize,Deserialize};

// 影子设备获取
// /shadow/get/${YourProductKey}/${YourDeviceName}
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShadowGetTopic {
	method: String,
	payload: Value,
	version: u32,
	timestamp: u64,
}

pub static TOPICS: Lazy<Vec<ALinkSubscribeTopic>> = Lazy::new(|| vec![
	ALinkSubscribeTopic::new("/shadow/get/+/+", TypeId::of::<ShadowGetTopic>()),
]);

pub enum ShadowRecv {
	ShadowGetTopic(ShadowGetTopic),
}

#[async_trait::async_trait]
impl crate::Executor for crate::shadow::Executor {
	async fn execute(&self, topic: &str, payload: &[u8]) -> crate::Result<()> {
		debug!("{} {}", topic, String::from_utf8_lossy(payload));
		for item in &*TOPICS {
			if !item.is_match(topic, &self.three.product_key, &self.three.device_name) {
				return Ok(());
			}
			let data = match item.payload_type_id {
				a if a == TypeId::of::<ShadowGetTopic>() => {
					ShadowRecv::ShadowGetTopic(serde_json::from_slice(&payload)?)
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