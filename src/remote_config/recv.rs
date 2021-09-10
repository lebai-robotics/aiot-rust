use crate::alink::alink_topic::ALinkSubscribeTopic;
use crate::{alink::AlinkResponse, Error};
use enum_iterator::IntoEnumIterator;
use enum_kinds::EnumKind;
use log::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use spin::Lazy;

/// 远程配置文件数据
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemoteConfigFileInfo {
	/// 配置文件的ID
	pub config_id: String,
	/// 配置文件大小，按字节计算。
	pub config_size: u64,
	/// 签名
	pub sign: String,
	/// 签名方法，仅支持Sha256
	pub sign_method: String,
	/// 存储配置文件的对象存储（OSS）地址
	pub url: String,
	/// 获取配置类型。 目前支持文件类型，取值：file
	pub get_type: String,
}

pub type RemoteConfigGetReply = AlinkResponse<RemoteConfigFileInfo>;
pub type RemoteConfigPush = AlinkResponse<RemoteConfigFileInfo>;

#[derive(Debug, EnumKind)]
#[enum_kind(RemoteConfigRecvKind, derive(Serialize, IntoEnumIterator, Deserialize))]
pub enum RemoteConfigRecv {
	/// 设备主动请求配置信息回应
	RemoteConfigGetReply(RemoteConfigGetReply),
	/// 配置推送
	RemoteConfigPush(RemoteConfigPush),
}

impl RemoteConfigRecvKind {
	pub fn match_kind(
		topic: &str,
		product_key: &str,
		device_name: &str,
	) -> Option<RemoteConfigRecvKind> {
		for item in RemoteConfigRecvKind::into_enum_iter() {
			let alink_topic = item.get_topic();
			if !alink_topic.is_match(topic, product_key, device_name) {
				continue;
			}
			return Some(item);
			// self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
		}
		None
	}
	pub fn to_payload(&self, payload: &[u8]) -> crate::Result<RemoteConfigRecv> {
		match *self {
			RemoteConfigRecvKind::RemoteConfigGetReply => Ok(RemoteConfigRecv::RemoteConfigGetReply(
				serde_json::from_slice(&payload)?,
			)),
			RemoteConfigRecvKind::RemoteConfigPush => Ok(RemoteConfigRecv::RemoteConfigPush(
				serde_json::from_slice(&payload)?,
			)),
		}
	}
	pub fn get_topic(&self) -> ALinkSubscribeTopic {
		match *self {
			RemoteConfigRecvKind::RemoteConfigGetReply => {
				ALinkSubscribeTopic::new_we("/sys/+/+/thing/config/get_reply")
			}
			RemoteConfigRecvKind::RemoteConfigPush => {
				ALinkSubscribeTopic::new_we("/sys/+/+/thing/config/push")
			}
		}
	}
}

#[async_trait::async_trait]
impl crate::Executor for crate::remote_config::Executor {
	async fn execute(&self, topic: &str, payload: &[u8]) -> crate::Result<()> {
		debug!("receive: {} {}", topic, String::from_utf8_lossy(payload));
		if let Some(kind) =
			RemoteConfigRecvKind::match_kind(topic, &self.three.product_key, &self.three.device_name)
		{
			let data = kind.to_payload(payload)?;
			self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
		} else {
			debug!("no match topic: {}", topic);
		}
		Ok(())
	}
}
