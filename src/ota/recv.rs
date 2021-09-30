use serde_json::Value;
use std::collections::HashMap;

use super::base::*;
use crate::alink::aiot_module::{get_aiot_json, ModuleRecvKind};
use crate::alink::alink_topic::ALinkSubscribeTopic;
use crate::alink::{AlinkRequest, AlinkResponse};
use crate::subdev::base::DeviceInfoId;
use enum_iterator::IntoEnumIterator;
use enum_kinds::EnumKind;
use serde::{Deserialize, Serialize};

pub type UpgradePackageRequest = AlinkResponse<Option<PackageData>, u128, String>;
pub type GetFirmwareReply = AlinkResponse<Option<PackageData>>;

#[derive(Debug, EnumKind)]
#[enum_kind(OTARecvKind, derive(Serialize, IntoEnumIterator, Deserialize))]
pub enum OTARecv {
	/// 物联网平台推送升级包
	UpgradePackageRequest(UpgradePackageRequest),
	/// 设备请求升级包响应
	GetFirmwareReply(GetFirmwareReply),
}

impl ModuleRecvKind for super::RecvKind {
	type Recv = super::Recv;
	fn to_payload(&self, payload: &[u8]) -> crate::Result<OTARecv> {
		let json_str = get_aiot_json(payload);
		match *self {
			Self::UpgradePackageRequest => Ok(Self::Recv::UpgradePackageRequest(
				serde_json::from_str(&json_str)?,
			)),
			Self::GetFirmwareReply => Ok(Self::Recv::GetFirmwareReply(serde_json::from_str(
				&json_str,
			)?)),
		}
	}
	fn get_topic(&self) -> ALinkSubscribeTopic {
		match *self {
			Self::UpgradePackageRequest => ALinkSubscribeTopic::new_we("/ota/device/upgrade/+/+"),
			Self::GetFirmwareReply => {
				ALinkSubscribeTopic::new("/sys/+/+/thing/ota/firmware/get_reply")
			}
		}
	}
}
