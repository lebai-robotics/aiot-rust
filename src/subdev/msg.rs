use crate::subdev::recv::DeviceInfoId;
use crate::alink::{SysAck, global_id_next};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::util::auth::sign_device;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
	pub device_name: String,
	pub product_key: String,
	pub sign: String,
	pub sign_method: String,
	pub timestamp: String,
	pub client_id: String,
	// 如果取值是true，则清理所有子设备离线时的消息，即所有未接收的QoS1消息将被清除。如果取值是false，则不清理子设备离线时的消息。
	pub clean_session: Option<String>,

}

impl DeviceInfo {
	pub fn new(device_name: String, product_key: String, clean_session: Option<bool>) -> Self {
		// client_id+device_name+product_key+timestamp;
		let client_id = format!("{}&{}", product_key, device_name);
		let start = SystemTime::now();
		let since_the_epoch = start.duration_since(UNIX_EPOCH)
			.expect("Time went backwards");
		let timestamp = since_the_epoch.as_millis();
		let sign = sign_device(&client_id, &device_name, &product_key, "ee1fe40b755a7034dadd0e47d69c83b7", timestamp);
		Self {
			device_name,
			product_key,
			sign,
			sign_method: "".to_string(),
			timestamp: timestamp.to_string(),
			client_id,
			clean_session: clean_session.map(|n| String::from(if n { "true" } else { "false" })),
		}
	}
}

pub type SubDevLogin = DeviceInfo;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubDevBatchLoginParams {
	pub device_list: Vec<DeviceInfo>,
}


#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubDevChangeTopologicalRelationNotifyParams {
	pub status: u32,
	//0-创建  1-删除 2-恢复禁用  8-禁用
	pub sub_list: Vec<DeviceInfoId>,
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

// 通知网关添加设备拓扑关系
// /sys/{productKey}/{deviceName}/thing/topo/add/notify
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubDevAddTopologicalRelationNotifyRequest {
	pub id: String,
	pub version: String,
	pub method: String,
	pub params: Vec<DeviceInfoId>,
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