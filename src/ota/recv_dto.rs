use serde_json::Value;
use std::collections::HashMap;


use crate::alink::{AlinkRequest, AlinkResponse};
use crate::alink_topic::ALinkSubscribeTopic;
use crate::subdev::base::DeviceInfoId;
use enum_iterator::IntoEnumIterator;
use enum_kinds::EnumKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, EnumKind)]
#[enum_kind(OTARecvKind, derive(Serialize, IntoEnumIterator, Deserialize))]
pub enum OTARecv {
	UpgradePackageRequest(UpgradePackageRequest),
	GetFirmwareReply(GetFirmwareReply),
}

impl OTARecvKind {
	pub fn match_kind(topic: &str, product_key: &str, device_name: &str) -> Option<OTARecvKind> {
		for item in OTARecvKind::into_enum_iter() {
			let alink_topic = item.get_topic();
			if !alink_topic.is_match(topic, product_key, device_name) {
				continue;
			}
			return Some(item);
		}
		None
	}
	pub fn to_payload(&self, payload: &[u8]) -> crate::Result<OTARecv> {
		match *self {
			Self::UpgradePackageRequest => Ok(OTARecv::UpgradePackageRequest(serde_json::from_slice(
				&payload,
			)?)),
			Self::GetFirmwareReply => Ok(OTARecv::GetFirmwareReply(
				serde_json::from_slice(&payload)?,
			)),
		}
	}
	pub fn get_topic(&self) -> ALinkSubscribeTopic {
		match *self {
			Self::UpgradePackageRequest => {
				ALinkSubscribeTopic::new("/ota/device/upgrade/+/+")
			}
			Self::GetFirmwareReply => {
				ALinkSubscribeTopic::new("/sys/+/+/thing/ota/firmware/get_reply")
			}
		}
	}
}

// 固件升级包信息
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PackageData {
	pub size: u64,
	pub version: String,
	pub is_diff: Option<u8>,
	pub url: String,
	pub md5: Option<String>,
	pub sign: String,
	pub sign_method: String,
	pub module: Option<String>,
	pub ext_data: Option<Value>,
}

pub type UpgradePackageRequest = AlinkResponse<PackageData>;
pub type GetFirmwareReply = AlinkResponse<PackageData>;
