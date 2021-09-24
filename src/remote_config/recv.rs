use crate::alink::aiot_module::{ModuleRecvKind, get_aiot_json};
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

pub type RemoteConfigGetReply = AlinkResponse<Option<RemoteConfigFileInfo>>;
pub type RemoteConfigPush = AlinkResponse<Option<RemoteConfigFileInfo>>;

#[derive(Debug, EnumKind)]
#[enum_kind(RemoteConfigRecvKind, derive(Serialize, IntoEnumIterator, Deserialize))]
pub enum RemoteConfigRecv {
	/// 设备主动请求配置信息回应
	RemoteConfigGetReply(RemoteConfigGetReply),
	/// 配置推送
	RemoteConfigPush(RemoteConfigPush),
}

impl ModuleRecvKind for super::RecvKind {
	type Recv = super::Recv;
	fn to_payload(&self, payload: &[u8]) -> crate::Result<Self::Recv> {
		let json_str = get_aiot_json(payload);
		match *self {
			Self::RemoteConfigGetReply => Ok(Self::Recv::RemoteConfigGetReply(serde_json::from_str(
				&json_str,
			)?)),
			Self::RemoteConfigPush => Ok(Self::Recv::RemoteConfigPush(serde_json::from_str(
				&json_str,
			)?)),
		}
	}
	fn get_topic(&self) -> ALinkSubscribeTopic {
		match *self {
			Self::RemoteConfigGetReply => {
				ALinkSubscribeTopic::new_we("/sys/+/+/thing/config/get_reply")
			}
			Self::RemoteConfigPush => ALinkSubscribeTopic::new_we("/sys/+/+/thing/config/push"),
		}
	}
}
