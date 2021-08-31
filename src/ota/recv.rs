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

// 云端下推的固件升级任务的描述信息, 包括url, 大小, 签名等
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UpgradePackage {
	pub size: u32,
	pub version: String,
	pub is_diff: bool,
	pub url: String,
	pub md5: String,
	pub sign: String,
	pub sign_method: String,
	pub module: String,
	pub ext_data: Value,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum RecvEnum {
	UpgradePackage(UpgradePackage),
	GetFirmwareReply(UpgradePackage),
}
