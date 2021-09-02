use crate::alink::{SysAck, global_id_next};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::util::auth::{sign_device, SIGN_METHOD};
use serde::{Deserialize, Serialize};
use crate::subdev::recv_dto::{SubDevMethodResponse};
use crate::subdev::base::*;

pub type SubDevLogin = DeviceInfo;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubDevBatchLoginParams {
	pub device_list: Vec<DeviceInfo>,
}


// 子设备上线
// /ext/session/${productKey}/${deviceName}/combine/login
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubDevLoginRequest {
	pub id: String,
	pub params: SubDevLogin,
}

// 子设备批量上线
// /ext/session/${productKey}/${deviceName}/combine/batch_login
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubDevBatchLoginRequest {
	pub id: String,
	pub params: SubDevBatchLoginParams,
}

// 子设备下线
// /ext/session/{productKey}/{deviceName}/combine/logout
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubDevLogoutRequest {
	pub id: u64,
	pub params: DeviceInfoId,
}

// 子设备批量下线
// /ext/session/{productKey}/{deviceName}/combine/batch_logout
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubDevBatchLogoutRequest {
	pub id: u64,
	pub params: Vec<DeviceInfoId>,
}

// 子设备操作，禁用，启用，删除
// /sys/{productKey}/{deviceName}/thing/disable
// /sys/{productKey}/{deviceName}/thing/enable
// /sys/{productKey}/{deviceName}/thing/delete
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubDevMethodRequest {
	pub id: String,
	pub version: String,
	pub method: String,
}

// 460	request parameter error	请求参数错误。
// 6402	topo relation cannot add by self	设备不能把自己添加为自己的子设备。
// 401	request auth error	签名校验授权失败。

// 添加拓扑关系
// /sys/{productKey}/{deviceName}/thing/topo/add
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubDevAddTopologicalRelationRequest {
	pub id: String,
	pub version: String,
	pub params: Vec<DeviceInfo>,
	pub sys: SysAck,
	pub method: String,
}

// 删除拓扑关系
// /sys/{productKey}/{deviceName}/thing/topo/delete
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubDevDeleteTopologicalRelationRequest {
	pub id: String,
	pub version: String,
	pub params: Vec<DeviceInfoId>,
	pub sys: SysAck,
	pub method: String,
}

// 获取拓扑关系
// /sys/{productKey}/{deviceName}/thing/topo/get
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubDevGetTopologicalRelationRequest {
	pub id: String,
	pub version: String,
	pub sys: SysAck,
	pub method: String,
}

// 发现设备信息上报
// /sys/{productKey}/{deviceName}/thing/list/found
pub type SubDevFoundReportRequest = SubDevDeleteTopologicalRelationRequest;


// 通知网关添加设备拓扑关系响应
pub type SubDevAddTopologicalRelationNotifyResponse = SubDevMethodResponse;

// 通知网关拓扑关系变化响应
pub struct SubDevChangeTopologicalRelationNotifyResponse {
	pub id: String,
	pub code: u32,
	pub message: String,
}