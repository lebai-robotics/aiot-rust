use crate::subdev::base::DeviceInfoId;
use crate::alink_topic::ALinkSubscribeTopic;
use spin::Lazy;
use std::any::TypeId;
use serde::{Deserialize, Serialize};

pub static TOPICS: Lazy<Vec<ALinkSubscribeTopic>> = Lazy::new(|| vec![
	ALinkSubscribeTopic::new("/ext/session/+/+/combine/login", TypeId::of::<SubDevLoginResponse>()),
	ALinkSubscribeTopic::new("/ext/session/+/+/combine/batch_login", TypeId::of::<SubDevBatchLoginResponse>()),
	ALinkSubscribeTopic::new("/ext/session/+/+/combine/logout", TypeId::of::<SubDevLogoutResponse>()),
	ALinkSubscribeTopic::new("/ext/session/+/+/combine/batch_logout", TypeId::of::<SubDevBatchLogoutResponse>()),
	ALinkSubscribeTopic::new("/sys/+/+/thing/disable", TypeId::of::<SubDevMethodResponse>()),
	ALinkSubscribeTopic::new("/sys/+/+/thing/enable", TypeId::of::<SubDevMethodResponse>()),
	ALinkSubscribeTopic::new("/sys/+/+/thing/delete", TypeId::of::<SubDevMethodResponse>()),
	ALinkSubscribeTopic::new("/sys/+/+/thing/topo/add", TypeId::of::<SubDevAddTopologicalRelationResponse>()),
	ALinkSubscribeTopic::new("/sys/+/+/thing/topo/delete", TypeId::of::<SubDevDeleteTopologicalRelationResponse>()),
	ALinkSubscribeTopic::new("/sys/+/+/thing/topo/get", TypeId::of::<SubDevGetTopologicalRelationResponse>()),
	ALinkSubscribeTopic::new("/sys/+/+/thing/list/found_reply", TypeId::of::<SubDevFoundReportResponse>()),
	ALinkSubscribeTopic::new("/sys/+/+/thing/topo/add/notify", TypeId::of::<SubDevAddTopologicalRelationNotifyRequest>()),
	ALinkSubscribeTopic::new("/sys/+/+/thing/topo/change", TypeId::of::<SubDevChangeTopologicalRelationNotifyRequest>()),
]);

pub enum SubDevRecv {
	SubDevLoginResponse(SubDevLoginResponse),
	SubDevBatchLoginResponse(SubDevBatchLoginResponse),
	SubDevLogoutResponse(SubDevLogoutResponse),
	SubDevBatchLogoutResponse(SubDevBatchLogoutResponse),
	SubDevMethodResponse(SubDevMethodResponse),
	SubDevAddTopologicalRelationResponse(SubDevAddTopologicalRelationResponse),
	SubDevDeleteTopologicalRelationResponse(SubDevDeleteTopologicalRelationResponse),
	SubDevGetTopologicalRelationResponse(SubDevGetTopologicalRelationResponse),
	SubDevDeviceReportResponse(SubDevFoundReportResponse),
	SubDevAddTopologicalRelationNotifyRequest(SubDevAddTopologicalRelationNotifyRequest),
	SubDevChangeTopologicalRelationNotifyRequest(SubDevChangeTopologicalRelationNotifyRequest),
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
#[derive(Deserialize, Serialize, Debug, Clone)]
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
pub type SubDevFoundReportResponse = SubDevMethodResponse;


// 通知网关添加设备拓扑关系
// /sys/{productKey}/{deviceName}/thing/topo/add/notify
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubDevAddTopologicalRelationNotifyRequest {
	pub id: String,
	pub version: String,
	pub method: String,
	pub params: Vec<DeviceInfoId>,
}


#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubDevChangeTopologicalRelationNotifyParams {
	pub status: u32,
	//0-创建  1-删除 2-恢复禁用  8-禁用
	pub sub_list: Vec<DeviceInfoId>,
}

// 通知网关拓扑关系变化
// /sys/{productKey}/{deviceName}/thing/topo/change
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubDevChangeTopologicalRelationNotifyRequest {
	pub id: String,
	pub version: String,
	pub method: String,
	pub params: SubDevChangeTopologicalRelationNotifyParams,
}

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