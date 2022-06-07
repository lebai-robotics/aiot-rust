use crate::alink::aiot_module::{get_aiot_json, ModuleRecvKind};
use crate::alink::alink_topic::ALinkSubscribeTopic;
use crate::alink::{AlinkRequest, AlinkResponse};
use crate::subdev::base::DeviceInfoId;
use enum_iterator::IntoEnumIterator;
use enum_kinds::EnumKind;
use serde::{Deserialize, Serialize};

use super::base::DeviceInfoWithSecret;

#[derive(Debug, EnumKind)]
#[enum_kind(SubDevRecvKind, derive(Serialize, IntoEnumIterator, Deserialize))]
pub enum SubDevRecv {
    /// 子设备上线响应
    SubDevLoginResponse(SubDevLoginResponse),
    /// 子设备批量上线响应
    SubDevBatchLoginResponse(SubDevBatchLoginResponse),
    /// 子设备下线响应
    SubDevLogoutResponse(SubDevLogoutResponse),
    /// 子设备批量下线响应
    SubDevBatchLogoutResponse(SubDevBatchLogoutResponse),
    /// 子设备禁用响应
    SubDevDisableResponse(SubDevDisableResponse),
    /// 子设备启用响应
    SubDevEnableResponse(SubDevEnableResponse),
    /// 子设备删除响应
    SubDevDeleteResponse(SubDevDeleteResponse),
    /// 添加拓扑关系响应
    SubDevAddTopologicalRelationResponse(SubDevAddTopologicalRelationResponse),
    /// 删除拓扑关系响应
    SubDevDeleteTopologicalRelationResponse(SubDevDeleteTopologicalRelationResponse),
    /// 获取拓扑关系响应
    SubDevGetTopologicalRelationResponse(SubDevGetTopologicalRelationResponse),
    /// 获取拓扑关系响应
    SubDevDeviceReportResponse(SubDevDeviceReportResponse),
    /// 通知网关添加设备拓扑关系
    SubDevAddTopologicalRelationNotifyRequest(SubDevAddTopologicalRelationNotifyRequest),
    /// 拓扑关系更改通知
    SubDevChangeTopologicalRelationNotifyRequest(SubDevChangeTopologicalRelationNotifyRequest),
    /// 子设备洞注册响应
    SubDevRegisterResponse(SubDevRegisterResponse),
}

impl ModuleRecvKind for super::RecvKind {
    type Recv = super::Recv;
    fn to_payload(&self, payload: &[u8], _: &Vec<String>) -> crate::Result<Self::Recv> {
        let json_str = get_aiot_json(payload);
        match *self {
            Self::SubDevLoginResponse => Ok(Self::Recv::SubDevLoginResponse(serde_json::from_str(
                &json_str,
            )?)),
            Self::SubDevBatchLoginResponse => Ok(Self::Recv::SubDevBatchLoginResponse(
                serde_json::from_str(&json_str)?,
            )),
            Self::SubDevLogoutResponse => Ok(Self::Recv::SubDevLogoutResponse(
                serde_json::from_str(&json_str)?,
            )),
            Self::SubDevBatchLogoutResponse => Ok(Self::Recv::SubDevBatchLogoutResponse(
                serde_json::from_str(&json_str)?,
            )),
            Self::SubDevDisableResponse => Ok(Self::Recv::SubDevDisableResponse(
                serde_json::from_str(&json_str)?,
            )),
            Self::SubDevEnableResponse => Ok(Self::Recv::SubDevEnableResponse(
                serde_json::from_str(&json_str)?,
            )),
            Self::SubDevDeleteResponse => Ok(Self::Recv::SubDevDeleteResponse(
                serde_json::from_str(&json_str)?,
            )),
            Self::SubDevAddTopologicalRelationResponse => Ok(
                Self::Recv::SubDevAddTopologicalRelationResponse(serde_json::from_str(&json_str)?),
            ),
            Self::SubDevDeleteTopologicalRelationResponse => {
                Ok(Self::Recv::SubDevDeleteTopologicalRelationResponse(
                    serde_json::from_str(&json_str)?,
                ))
            }
            Self::SubDevGetTopologicalRelationResponse => Ok(
                Self::Recv::SubDevGetTopologicalRelationResponse(serde_json::from_str(&json_str)?),
            ),
            Self::SubDevDeviceReportResponse => Ok(Self::Recv::SubDevDeviceReportResponse(
                serde_json::from_str(&json_str)?,
            )),
            Self::SubDevAddTopologicalRelationNotifyRequest => {
                Ok(Self::Recv::SubDevAddTopologicalRelationNotifyRequest(
                    serde_json::from_str(&json_str)?,
                ))
            }
            Self::SubDevChangeTopologicalRelationNotifyRequest => {
                Ok(Self::Recv::SubDevChangeTopologicalRelationNotifyRequest(
                    serde_json::from_str(&json_str)?,
                ))
            }
            Self::SubDevRegisterResponse => Ok(Self::Recv::SubDevRegisterResponse(
                serde_json::from_str(&json_str)?,
            )),
        }
    }
    fn get_topic(&self) -> ALinkSubscribeTopic {
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
            Self::SubDevRegisterResponse => {
                ALinkSubscribeTopic::new("/sys/+/+/thing/sub/register_reply")
            }
        }
    }
}

// 子设备上线响应
pub type SubDevLoginResponse = AlinkResponse<DeviceInfoId>;

// 子设备批量上线响应
pub type SubDevBatchLoginResponse = AlinkResponse<Vec<DeviceInfoId>>;

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
pub type SubDevAddTopologicalRelationResponse = AlinkResponse<Option<Vec<DeviceInfoId>>>;

// 删除拓扑关系响应
pub type SubDevDeleteTopologicalRelationResponse = AlinkResponse<Option<Vec<DeviceInfoId>>>;

// 获取拓扑关系响应
pub type SubDevGetTopologicalRelationResponse = AlinkResponse<Option<Vec<DeviceInfoId>>>;

// 发现设备上报响应
pub type SubDevDeviceReportResponse = AlinkResponse;

// 通知网关添加设备拓扑关系
pub type SubDevAddTopologicalRelationNotifyRequest = AlinkRequest<Option<Vec<DeviceInfoId>>>;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubDevChangeTopologicalRelationNotifyParams {
    pub status: u32,
    //0-创建  1-删除 2-恢复禁用  8-禁用
    pub sub_list: Vec<DeviceInfoId>,
}

// 通知网关拓扑关系变化
pub type SubDevChangeTopologicalRelationNotifyRequest =
    AlinkRequest<SubDevChangeTopologicalRelationNotifyParams>;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubDevRegisterResult {
    pub iot_id: String,
    pub device_secret: String,
    pub device_name: String,
    pub product_key: String,
}

impl From<SubDevRegisterResult> for DeviceInfoWithSecret {
    fn from(r: SubDevRegisterResult) -> Self {
        DeviceInfoWithSecret {
            device_name: r.device_name,
            product_key: r.product_key,
            device_secret: r.device_secret,
        }
    }
}

// 子设备动态注册响应
pub type SubDevRegisterResponse = AlinkResponse<Option<Vec<SubDevRegisterResult>>>;

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
