use crate::alink::{AlinkRequest, AlinkResponse};
use crate::alink_topic::ALinkSubscribeTopic;
use crate::subdev::base::DeviceInfoId;
use enum_iterator::IntoEnumIterator;
use enum_kinds::EnumKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, EnumKind)]
#[enum_kind(SubDevRecvKind, derive(Serialize, IntoEnumIterator, Deserialize))]
pub enum SubDevRecv {
	SubDevLoginResponse(SubDevLoginResponse),
	SubDevBatchLoginResponse(SubDevBatchLoginResponse),
	SubDevLogoutResponse(SubDevLogoutResponse),
	SubDevBatchLogoutResponse(SubDevBatchLogoutResponse),
	SubDevDisableResponse(SubDevDisableResponse),
	SubDevEnableResponse(SubDevEnableResponse),
	SubDevDeleteResponse(SubDevDeleteResponse),
	SubDevAddTopologicalRelationResponse(SubDevAddTopologicalRelationResponse),
	SubDevDeleteTopologicalRelationResponse(SubDevDeleteTopologicalRelationResponse),
	SubDevGetTopologicalRelationResponse(SubDevGetTopologicalRelationResponse),
	SubDevDeviceReportResponse(SubDevDeviceReportResponse),
	SubDevAddTopologicalRelationNotifyRequest(SubDevAddTopologicalRelationNotifyRequest),
	SubDevChangeTopologicalRelationNotifyRequest(SubDevChangeTopologicalRelationNotifyRequest),
}

impl SubDevRecvKind {
	pub fn match_kind(topic: &str, product_key: &str, device_name: &str) -> Option<SubDevRecvKind> {
		for item in SubDevRecvKind::into_enum_iter() {
			let alink_topic = item.get_topic();
			if !alink_topic.is_match(topic, product_key, device_name) {
				continue;
			}
			return Some(item);
			// self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
		}
		None
	}
	pub fn to_payload(&self, payload: &[u8]) -> crate::Result<SubDevRecv> {
		match *self {
			Self::SubDevLoginResponse => Ok(SubDevRecv::SubDevLoginResponse(serde_json::from_slice(
				&payload,
			)?)),
			Self::SubDevBatchLoginResponse => Ok(SubDevRecv::SubDevBatchLoginResponse(
				serde_json::from_slice(&payload)?,
			)),
			Self::SubDevLogoutResponse => Ok(SubDevRecv::SubDevLogoutResponse(
				serde_json::from_slice(&payload)?,
			)),
			Self::SubDevBatchLogoutResponse => Ok(SubDevRecv::SubDevBatchLogoutResponse(
				serde_json::from_slice(&payload)?,
			)),
			Self::SubDevDisableResponse => Ok(SubDevRecv::SubDevDisableResponse(
				serde_json::from_slice(&payload)?,
			)),
			Self::SubDevEnableResponse => Ok(SubDevRecv::SubDevEnableResponse(
				serde_json::from_slice(&payload)?,
			)),
			Self::SubDevDeleteResponse => Ok(SubDevRecv::SubDevDeleteResponse(
				serde_json::from_slice(&payload)?,
			)),
			Self::SubDevAddTopologicalRelationResponse => Ok(
				SubDevRecv::SubDevAddTopologicalRelationResponse(serde_json::from_slice(&payload)?),
			),
			Self::SubDevDeleteTopologicalRelationResponse => Ok(
				SubDevRecv::SubDevDeleteTopologicalRelationResponse(serde_json::from_slice(&payload)?),
			),
			Self::SubDevGetTopologicalRelationResponse => Ok(
				SubDevRecv::SubDevGetTopologicalRelationResponse(serde_json::from_slice(&payload)?),
			),
			Self::SubDevDeviceReportResponse => Ok(SubDevRecv::SubDevDeviceReportResponse(
				serde_json::from_slice(&payload)?,
			)),
			Self::SubDevAddTopologicalRelationNotifyRequest => {
				Ok(SubDevRecv::SubDevAddTopologicalRelationNotifyRequest(
					serde_json::from_slice(&payload)?,
				))
			}
			Self::SubDevChangeTopologicalRelationNotifyRequest => {
				Ok(SubDevRecv::SubDevChangeTopologicalRelationNotifyRequest(
					serde_json::from_slice(&payload)?,
				))
			}
		}
	}
	pub fn get_topic(&self) -> ALinkSubscribeTopic {
		match *self {
			Self::SubDevLoginResponse => {
				ALinkSubscribeTopic::new("/ext/session/+/+/combine/login_reply")
			}
			Self::SubDevBatchLoginResponse => {
				ALinkSubscribeTopic::new("/ext/session/+/+/combine/batch_login_reply")
			}
			Self::SubDevLogoutResponse => {
				ALinkSubscribeTopic::new("/ext/session/+/+/combine/logout_reply")
			}
			Self::SubDevBatchLogoutResponse => {
				ALinkSubscribeTopic::new("/ext/session/+/+/combine/batch_logout_reply")
			}
			Self::SubDevDisableResponse => ALinkSubscribeTopic::new("/sys/+/+/thing/disable"),
			Self::SubDevEnableResponse => ALinkSubscribeTopic::new("/sys/+/+/thing/enable"),
			Self::SubDevDeleteResponse => ALinkSubscribeTopic::new("/sys/+/+/thing/delete"),
			Self::SubDevAddTopologicalRelationResponse => {
				ALinkSubscribeTopic::new("/sys/+/+/thing/topo/add_reply")
			}
			Self::SubDevDeleteTopologicalRelationResponse => {
				ALinkSubscribeTopic::new("/sys/+/+/thing/topo/delete_reply")
			}
			Self::SubDevGetTopologicalRelationResponse => {
				ALinkSubscribeTopic::new("/sys/+/+/thing/topo/get_reply")
			}
			Self::SubDevDeviceReportResponse => {
				ALinkSubscribeTopic::new("/sys/+/+/thing/list/found_reply")
			}
			Self::SubDevAddTopologicalRelationNotifyRequest => {
				ALinkSubscribeTopic::new("/sys/+/+/thing/topo/add/notify")
			}
			Self::SubDevChangeTopologicalRelationNotifyRequest => {
				ALinkSubscribeTopic::new("/sys/+/+/thing/topo/change")
			}
		}
	}
}

// 子设备上线响应
pub type SubDevLoginResponse = AlinkResponse<DeviceInfoId>;

// 子设备批量上线响应
pub type SubDevBatchLoginResponse =AlinkResponse<Vec<DeviceInfoId>>;

// 460	request parameter error	请求参数错误。
// 520	device no session	子设备会话不存在。

// 子设备下线响应
pub type SubDevLogoutResponse = AlinkResponse<DeviceInfoId>;

// 子设备批量下线响应
pub type SubDevBatchLogoutResponse = AlinkResponse<Vec<DeviceInfoId>>;

// 子设备禁用
pub type SubDevDisableResponse = AlinkRequest;

// 子设备启用
pub type SubDevEnableResponse = AlinkRequest;

// 子设备删除
pub type SubDevDeleteResponse = AlinkRequest;

// 添加拓扑关系响应
pub type SubDevAddTopologicalRelationResponse = AlinkResponse<Vec<DeviceInfoId>>;

// 删除拓扑关系响应
pub type SubDevDeleteTopologicalRelationResponse = AlinkResponse<Vec<DeviceInfoId>>;

// 获取拓扑关系响应
pub type SubDevGetTopologicalRelationResponse = AlinkResponse<Vec<DeviceInfoId>>;

// 发现设备上报响应
pub type SubDevDeviceReportResponse = AlinkResponse;

// 通知网关添加设备拓扑关系
pub type SubDevAddTopologicalRelationNotifyRequest = AlinkRequest<Vec<DeviceInfoId>>;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubDevChangeTopologicalRelationNotifyParams {
	pub status: u32,
	//0-创建  1-删除 2-恢复禁用  8-禁用
	pub sub_list: Vec<DeviceInfoId>,
}

// 通知网关拓扑关系变化
pub type SubDevChangeTopologicalRelationNotifyRequest = AlinkRequest<SubDevChangeTopologicalRelationNotifyParams>;


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
