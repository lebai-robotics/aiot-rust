use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// data-model模块接收消息的结构体
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OTARecv {
	/// 消息所属设备的product_key, 不配置则默认使用MQTT模块配置的product_key
	pub product_key: String,
	/// 消息所属设备的device_name, 不配置则默认使用MQTT模块配置的device_name
	pub device_name: String,
	/// 接收消息数据
	pub data: RecvEnum,
}

// 固件升级包信息
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PackageData {
	pub size: u64,
	pub version: String,
	pub is_diff: Option<bool>,
	pub url: String,
	pub md5: Option<String>,
	pub sign: String,
	pub sign_method: String,
	pub module: Option<String>,
	pub ext_data: Option<Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UpgradePackageRequest {
	pub code: String,
	pub data: PackageData,
	pub id: u64,
	pub message: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GetFirmwareReply {
	pub code: u32,
	pub id: String,
	pub data: Option<PackageData>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum RecvEnum {
	UpgradePackageRequest(UpgradePackageRequest),
	GetFirmwareReply(GetFirmwareReply),
}
