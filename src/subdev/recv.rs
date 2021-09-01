use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub enum SubDevRecv {}
/*
#[derive(TryFromPrimitive)]
#[repr(u32)]
enum SubDevLoginResponseCode {
	// 请求参数错误。
	RequestParameterError = 460,
	// 单个设备认证过于频繁被限流。
	RateLimitLimit = 429,
	// 网关下同时在线子设备过多。
	TooManySubDevicesUnderGateway = 428,
	// 网关和子设备没有拓扑关系。
	TopologicalRelationNotExist = 6401,
	// 子设备不存在。
	DeviceNotFound = 6100,
	// 子设备已被删除。
	DeviceDeleted = 521,
	// 子设备已被禁用。
	DeviceForbidden = 522,
	// 子设备密码或者签名错误。
	InvalidSign = 6287,
}*/

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfoId {
	pub device_name: String,
	pub product_key: String,
}

// 子设备上线响应
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubDevLoginResponse {
	pub id: String,
	pub code: u32,
	pub message: String,
	pub data: DeviceInfoId,
}

// 子设备批量上线响应
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubDevBatchLoginResponse {
	pub id: String,
	pub code: String,
	pub message: String,
	pub data: Vec<DeviceInfoId>,
}


// 460	request parameter error	请求参数错误。
// 520	device no session	子设备会话不存在。

// 子设备下线响应
pub type SubDevLogoutResponse = SubDevLoginResponse;

// 子设备批量下线响应
pub type SubDevBatchLogoutResponse = SubDevBatchLoginResponse;

// 子设备操作，禁用，启用，删除
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubDevMethodResponse {
	pub id: String,
	pub code: u32,
}

// 添加拓扑关系响应
pub struct SubDevAddTopologicalRelationResponse {
	pub id: String,
	pub code: u64,
	pub data: Vec<DeviceInfoId>,
}

// 删除拓扑关系响应
pub type SubDevDeleteTopologicalRelationResponse = SubDevAddTopologicalRelationResponse;

// 获取拓扑关系响应
pub type SubDevGetTopologicalRelationResponse = SubDevAddTopologicalRelationResponse;

// 发现设备上报响应
pub type SubDevDeviceReportResponse = SubDevMethodResponse;

// 通知网关添加设备拓扑关系响应
pub type SubDevAddTopologicalRelationNotifyResponse = SubDevMethodResponse;

// 通知网关拓扑关系变化响应
pub struct SubDevChangeTopologicalRelationNotifyResponse {
	pub id: String,
	pub code: u32,
	pub message: String,
}