use crate::alink::aiot_module::ModuleRecvKind;
use crate::alink::{aiot_module::get_aiot_json, alink_topic::ALinkSubscribeTopic};
use crate::Error;
use enum_iterator::IntoEnumIterator;
use enum_kinds::EnumKind;
use log::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use spin::Lazy;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShadowGetTopic {
    pub method: String,
    pub payload: Value,
    pub timestamp: u64,
}

#[derive(Debug, EnumKind)]
#[enum_kind(ShadowRecvKind, derive(Serialize, IntoEnumIterator, Deserialize))]
pub enum ShadowRecv {
    /// 设备主动获取影子内容响应
    ShadowGetTopic(ShadowGetTopic),
}

impl ModuleRecvKind for super::RecvKind {
    type Recv = super::Recv;
    fn to_payload(&self, payload: &[u8], _: &Vec<String>) -> crate::Result<ShadowRecv> {
        let json_str = get_aiot_json(payload);
        match *self {
            Self::ShadowGetTopic => {
                Ok(Self::Recv::ShadowGetTopic(serde_json::from_str(&json_str)?))
            }
        }
    }

    fn get_topic(&self) -> ALinkSubscribeTopic {
        match *self {
            Self::ShadowGetTopic => ALinkSubscribeTopic::new_we("/shadow/get/+/+"),
        }
    }
}
