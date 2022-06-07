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
#[enum_kind(RecvEnumKind, derive(Serialize, IntoEnumIterator, Deserialize))]
pub enum RecvEnum {
    /// 上报属性/实践后服务器返回的应答消息
    EventPostReply(GenericReply),
    /// 服务器下发的属性设置消息
    ServicePropertySet(PropertySet),
    /// 服务器下发的异步服务调用消息
    Service(AsyncServiceInvoke),
    /// 服务器下发的同步服务调用消息
    RrpcService(SyncServiceInvoke),
    /// 服务器下发的物模型二进制数据
    ModelDownRaw(RawData),
    /// 服务器对设备上报的二进制数据应答
    ModelUpRawReply(RawData),
    /// 服务器下发的二进制格式的同步服务调用消息
    RrpcDownRaw(RawServiceInvoke),
    PropertyDesiredGetReply(GenericReply),
    PropertyDesiredDeleteReply(GenericReply),
    PropertyBatchPostReply(GenericReply),
    PropertyHistoryPostReply(GenericReply),
}

impl ModuleRecvKind for super::RecvKind {
    type Recv = super::Recv;

    fn to_payload(&self, payload: &[u8], caps: &Vec<String>) -> crate::Result<RecvEnum> {
        let s = get_aiot_json(payload);
        match *self {
            Self::EventPostReply => Ok(Self::Recv::EventPostReply(GenericReply::s(&s, &caps)?)),
            Self::PropertyDesiredGetReply => Ok(Self::Recv::PropertyDesiredGetReply(
                GenericReply::s(&s, &caps)?,
            )),
            Self::PropertyDesiredDeleteReply => Ok(Self::Recv::PropertyDesiredDeleteReply(
                GenericReply::s(&s, &caps)?,
            )),
            Self::PropertyBatchPostReply => Ok(Self::Recv::PropertyBatchPostReply(
                GenericReply::s(&s, &caps)?,
            )),
            Self::PropertyHistoryPostReply => Ok(Self::Recv::PropertyHistoryPostReply(
                GenericReply::s(&s, &caps)?,
            )),
            Self::ServicePropertySet => {
                let payload: AlinkRequest<Value> = serde_json::from_slice(payload)?;
                let data = PropertySet {
                    msg_id: payload.msg_id(),
                    params: payload.params,
                };
                Ok(Self::Recv::ServicePropertySet(data))
            }
            Self::Service => {
                let payload: AlinkRequest<Value> = serde_json::from_slice(payload)?;
                let data = AsyncServiceInvoke {
                    msg_id: payload.msg_id(),
                    service_id: (&caps[3]).to_string(),
                    params: payload.params,
                };
                Ok(Self::Recv::Service(data))
            }
            Self::RrpcService => {
                let payload: AlinkRequest<Value> = serde_json::from_slice(payload)?;
                let data = SyncServiceInvoke {
                    rrpc_id: (&caps[1]).to_string(),
                    msg_id: payload.msg_id(),
                    service_id: (&caps[4]).to_string(),
                    params: payload.params,
                };
                Ok(Self::Recv::RrpcService(data))
            }
            Self::ModelDownRaw => {
                let data = RawData {
                    data: payload.to_vec(),
                };
                Ok(Self::Recv::ModelDownRaw(data))
            }
            Self::ModelUpRawReply => {
                let data = RawData {
                    data: payload.to_vec(),
                };
                Ok(Self::Recv::ModelUpRawReply(data))
            }
            Self::RrpcDownRaw => {
                let data = RawServiceInvoke {
                    rrpc_id: (&caps[1]).to_string(),
                    data: payload.to_vec(),
                };
                Ok(Self::Recv::RrpcDownRaw(data))
            }
        }
    }

    fn get_topic(&self) -> ALinkSubscribeTopic {
        match *self {
            Self::EventPostReply => ALinkSubscribeTopic::new_with_regex(
                "/sys/+/+/thing/event/+/post_reply",
                Regex::new(r"/sys/(.*)/(.*)/thing/event/(.*)/post_reply").unwrap(),
            ),
            Self::ServicePropertySet => {
                ALinkSubscribeTopic::new("/sys/+/+/thing/service/property/set")
            }
            Self::Service => ALinkSubscribeTopic::new_with_regex(
                "/sys/+/+/thing/service/+",
                Regex::new(r"/sys/(.*)/(.*)/thing/service/(.*)").unwrap(),
            ),
            Self::RrpcService => ALinkSubscribeTopic {
                topic: "/ext/rrpc/+/sys/+/+/thing/service/+",
                reg: Regex::new(r"/ext/rrpc/(.*)/sys/(.*)/(.*)/thing/service/(.*)").unwrap(),
                offset: 1,
            },
            Self::ModelDownRaw => ALinkSubscribeTopic::new("/sys/+/+/thing/model/down_raw"),
            Self::ModelUpRawReply => ALinkSubscribeTopic::new("/sys/+/+/thing/model/up_raw_reply"),
            Self::RrpcDownRaw => ALinkSubscribeTopic {
                topic: "/ext/rrpc/+/sys/+/+/thing/model/down_raw",
                reg: Regex::new(r"/ext/rrpc/(.*)/sys/(.*)/(.*)/thing/model/down_raw").unwrap(),
                offset: 1,
            },
            Self::PropertyDesiredGetReply => {
                ALinkSubscribeTopic::new("/sys/+/+/thing/property/desired/get_reply")
            }
            Self::PropertyDesiredDeleteReply => {
                ALinkSubscribeTopic::new("/sys/+/+/thing/property/desired/delete_reply")
            }
            Self::PropertyBatchPostReply => {
                ALinkSubscribeTopic::new("/sys/+/+/thing/event/property/batch/post_reply")
            }
            Self::PropertyHistoryPostReply => {
                ALinkSubscribeTopic::new("/sys/+/+/thing/event/property/history/post_reply")
            }
        }
    }
}

impl GenericReply {
    pub fn s(s: &str, caps: &Vec<String>) -> crate::Result<Self> {
        let payload: AlinkResponse<Value> = serde_json::from_str(&s)?;
        let data = GenericReply {
            msg_id: payload.msg_id(),
            code: payload.code,
            data: payload.data,
            message: payload.message.unwrap_or("".to_string()),
        };
        Ok(data)
    }
}
