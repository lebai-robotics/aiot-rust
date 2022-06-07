use super::base::*;
use crate::alink::aiot_module::{get_aiot_json, ModuleRecvKind};
use crate::alink::alink_topic::ALinkSubscribeTopic;
use crate::alink::{AlinkRequest, AlinkResponse, SimpleResponse};
use crate::{Error, Result, ThreeTuple};
use enum_iterator::IntoEnumIterator;
use enum_kinds::EnumKind;
use regex::Regex;
use rumqttc::{AsyncClient, QoS};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::TypeId;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Debug, EnumKind)]
#[enum_kind(DMRecvKind, derive(Serialize, IntoEnumIterator, Deserialize))]
pub enum DMRecv {
    /// 上报属性/实践后服务器返回的应答消息
    EventPostReply(DataModelRecv<GenericReply>),
    // /// 服务器下发的属性设置消息
    // PropertySet(PropertySet),
    // /// 服务器下发的异步服务调用消息
    // AsyncServiceInvoke(AsyncServiceInvoke),
    // /// 服务器下发的同步服务调用消息
    // SyncServiceInvoke(SyncServiceInvoke),
    // /// 服务器对设备上报的二进制数据应答
    // RawDataReply(RawData),
    // /// 服务器下发的物模型二进制数据
    // RawData(RawData),
    // /// 服务器下发的二进制格式的同步服务调用消息
    // RawSyncServiceInvoke(RawServiceInvoke),
}

impl ModuleRecvKind for super::RecvKind {
    type Recv = super::Recv;

    fn to_payload(&self, payload: &[u8], caps: &Vec<String>) -> crate::Result<DMRecv> {
        let s = get_aiot_json(payload);
        match *self {
            Self::EventPostReply => {
                let payload: AlinkResponse<Value> = serde_json::from_slice(payload)?;

                let data = GenericReply {
                    msg_id: payload.msg_id(),
                    code: payload.code,
                    data: payload.data,
                    message: payload.message.unwrap_or("".to_string()),
                };
                let data = DataModelRecv::new(&caps[1], &caps[2], data);
                Ok(Self::Recv::EventPostReply(data))
            }
        }
    }

    fn get_topic(&self) -> ALinkSubscribeTopic {
        match *self {
            Self::EventPostReply => ALinkSubscribeTopic::new_with_regex(
                "/sys/+/+/thing/event/+/post_reply",
                Regex::new(r"/sys/(.*)/(.*)/thing/event/(.*)/post_reply").unwrap(),
            ),
        }
    }
}
