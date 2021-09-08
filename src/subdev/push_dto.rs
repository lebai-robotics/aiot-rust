use crate::alink::{AlinkRequest, SysAck};
use crate::subdev::base::*;
use serde::{Deserialize, Serialize};

pub type SubDevLogin = DeviceInfo;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubDevBatchLoginParams {
	pub device_list: Vec<DeviceInfo>,
}

// 子设备上线
// /ext/session/${productKey}/${deviceName}/combine/login
pub type SubDevLoginRequest = AlinkRequest<SubDevLogin>;

// 子设备批量上线
// /ext/session/${productKey}/${deviceName}/combine/batch_login
pub type SubDevBatchLoginRequest = AlinkRequest<SubDevBatchLoginParams>;

// 子设备下线
// /ext/session/{productKey}/{deviceName}/combine/logout
pub type SubDevLogoutRequest = AlinkRequest<DeviceInfoId>;

// 子设备批量下线
// /ext/session/{productKey}/{deviceName}/combine/batch_logout
pub type SubDevBatchLogoutRequest = AlinkRequest<Vec<DeviceInfoId>>;

// 添加拓扑关系
// /sys/{productKey}/{deviceName}/thing/topo/add
pub type SubDevAddTopologicalRelationRequest = AlinkRequest<Vec<DeviceInfoNoCS>>;

// 删除拓扑关系
// /sys/{productKey}/{deviceName}/thing/topo/delete
pub type SubDevDeleteTopologicalRelationRequest = AlinkRequest<Vec<DeviceInfoId>>;

// 获取拓扑关系
// /sys/{productKey}/{deviceName}/thing/topo/get
pub type SubDevGetTopologicalRelationRequest  =AlinkRequest;

// 发现设备信息上报
// /sys/{productKey}/{deviceName}/thing/list/found
pub type SubDevFoundReportRequest = AlinkRequest<Vec<DeviceInfoId>>;

// 通知网关添加设备拓扑关系响应
pub type SubDevAddTopologicalRelationNotifyResponse = AlinkRequest;

// 通知网关拓扑关系变化响应
pub type SubDevChangeTopologicalRelationNotifyResponse = AlinkRequest;

// 子设备禁用
pub type SubDevDisableResponse = AlinkRequest;
// 子设备启用
pub type SubDevEnableResponse = AlinkRequest;
// 子设备删除
pub type SubDevDeleteResponse = AlinkRequest;

// 子设备动态注册
pub type SubDevRegisterRequest = AlinkRequest<Vec<DeviceInfoId>>;
