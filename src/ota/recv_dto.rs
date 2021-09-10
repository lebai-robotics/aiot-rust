use serde_json::Value;
use std::collections::HashMap;

use crate::alink::alink_topic::ALinkSubscribeTopic;
use crate::alink::{AlinkRequest, AlinkResponse};
use crate::subdev::base::DeviceInfoId;
use enum_iterator::IntoEnumIterator;
use enum_kinds::EnumKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, EnumKind)]
#[enum_kind(OTARecvKind, derive(Serialize, IntoEnumIterator, Deserialize))]
pub enum OTARecv {
	/// 物联网平台推送升级包
	UpgradePackageRequest(UpgradePackageRequest),
	/// 设备请求升级包响应
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
		let json_str = String::from_utf8_lossy(&payload).replace(",\"data\":{},", ",\"data\":null,");
		match *self {
			Self::UpgradePackageRequest => {
				Ok(OTARecv::UpgradePackageRequest(serde_json::from_str(&json_str)?))
			},
			Self::GetFirmwareReply => {
				Ok(OTARecv::GetFirmwareReply(serde_json::from_str(&json_str)?))
			},
		}
	}
	pub fn get_topic(&self) -> ALinkSubscribeTopic {
		match *self {
			Self::UpgradePackageRequest => ALinkSubscribeTopic::new_we("/ota/device/upgrade/+/+"),
			Self::GetFirmwareReply => {
				ALinkSubscribeTopic::new("/sys/+/+/thing/ota/firmware/get_reply")
			}
		}
	}
}

/// 固件升级包信息
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PackageData {
	/// 大小
	pub size: u64,
	/// 版本
	pub version: String,
	/// 是否使用了差分升级
	pub is_diff: Option<u8>,
	/// 包Url
	pub url: String,
	/// MD5
	pub md5: Option<String>,
	/// 签名
	pub sign: String,
	/// 签名方法
	pub sign_method: String,
	/// 升级包所属模块名
	pub module: Option<String>,
	/// 升级批次标签列表和推送给设备的自定义信息。
	/// _package_udi表示自定义信息的字段。
	/// 单个标签格式："key":"value"。
	pub ext_data: Option<Value>,
}

pub type UpgradePackageRequest = AlinkResponse<Option<PackageData>, u128, String>;
pub type GetFirmwareReply = AlinkResponse<Option<PackageData>>;
