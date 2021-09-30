use std::any::TypeId;

use enum_iterator::IntoEnumIterator;
use enum_kinds::EnumKind;
use log::*;
use serde::{Deserialize, Serialize};

use crate::alink::aiot_module::{get_aiot_json, ModuleRecvKind};
use crate::alink::alink_topic::ALinkSubscribeTopic;
use crate::alink::{AlinkResponse, SimpleResponse};
use crate::Error;

// 标签信息上报响应
// /sys/{productKey}/{deviceName}/thing/deviceinfo/update_reply
pub type DeviceInfoUpdateResponse = SimpleResponse;

// 标签信息删除响应
// /sys/{productKey}/{deviceName}/thing/deviceinfo/delete_replly
pub type DeviceInfoDeleteResponse = SimpleResponse;

#[derive(Debug, EnumKind)]
#[enum_kind(TagRecvKind, derive(Serialize, IntoEnumIterator, Deserialize))]
pub enum TagRecv {
    /// 标签信息上报响应
    DeviceInfoUpdateResponse(DeviceInfoUpdateResponse),
    /// 标签信息删除响应
    DeviceInfoDeleteResponse(DeviceInfoDeleteResponse),
}

impl ModuleRecvKind for super::RecvKind {
    type Recv = super::Recv;

    fn to_payload(&self, payload: &[u8]) -> crate::Result<TagRecv> {
        let json_str = get_aiot_json(payload);
        match *self {
            Self::DeviceInfoUpdateResponse => Ok(Self::Recv::DeviceInfoUpdateResponse(
                serde_json::from_str(&json_str)?,
            )),
            Self::DeviceInfoDeleteResponse => Ok(Self::Recv::DeviceInfoDeleteResponse(
                serde_json::from_str(&json_str)?,
            )),
        }
    }

    fn get_topic(&self) -> ALinkSubscribeTopic {
        match *self {
            Self::DeviceInfoUpdateResponse => {
                ALinkSubscribeTopic::new("/sys/+/+/thing/deviceinfo/update_reply")
            }
            Self::DeviceInfoDeleteResponse => {
                ALinkSubscribeTopic::new("/sys/+/+/thing/deviceinfo/delete_reply")
            }
        }
    }
}
