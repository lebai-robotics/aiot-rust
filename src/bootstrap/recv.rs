
use crate::alink::aiot_module::{ModuleRecvKind, get_aiot_json};
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

impl ModuleRecvKind for super::RecvKind {
	type Recv =super::Recv;

	fn get_topic(&self) -> ALinkSubscribeTopic {
		match *self {
			Self::BootstrapNotify => {
				ALinkSubscribeTopic::new_we("/sys/+/+/thing/bootstrap/notify")
			}
		}
	}

	fn to_payload(&self, payload: &[u8]) -> crate::Result<Self::Recv> {
		let json_str = get_aiot_json(payload);
		match *self {
			Self::BootstrapNotify => Ok(Self::Recv::BootstrapNotify(
				serde_json::from_str(&json_str)?,
			)),
		}
    }
}