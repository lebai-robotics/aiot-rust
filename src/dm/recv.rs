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
    EventPostReply(DMGenericReply),
    /// 服务器下发的属性设置消息
    ServicePropertySet(DataModelRecv<PropertySet>),
    /// 服务器下发的异步服务调用消息
    Service(DataModelRecv<AsyncServiceInvoke>),
    /// 服务器下发的同步服务调用消息
    RrpcService(DataModelRecv<SyncServiceInvoke>),
    /// 服务器下发的物模型二进制数据
    ModelDownRaw(DataModelRecv<RawData>),
    /// 服务器对设备上报的二进制数据应答
    ModelUpRawReply(DataModelRecv<RawData>),
    /// 服务器下发的二进制格式的同步服务调用消息
    RrpcDownRaw(DataModelRecv<RawServiceInvoke>),
    PropertyDesiredGetReply(DMGenericReply),
    PropertyDesiredDeleteReply(DMGenericReply),
    PropertyBatchPostReply(DMGenericReply),
    PropertyHistoryPostReply(DMGenericReply),
}

impl ModuleRecvKind for super::RecvKind {
    type Recv = super::Recv;

    fn to_payload(&self, payload: &[u8], caps: &Vec<String>) -> crate::Result<DMRecv> {
        let s = get_aiot_json(payload);
        match *self {
            Self::EventPostReply => Ok(Self::Recv::EventPostReply(DMGenericReply::s(&s, &caps)?)),
            Self::PropertyDesiredGetReply => Ok(Self::Recv::PropertyDesiredGetReply(
                DMGenericReply::s(&s, &caps)?,
            )),
            Self::PropertyDesiredDeleteReply => Ok(Self::Recv::PropertyDesiredDeleteReply(
                DMGenericReply::s(&s, &caps)?,
            )),
            Self::PropertyBatchPostReply => Ok(Self::Recv::PropertyBatchPostReply(
                DMGenericReply::s(&s, &caps)?,
            )),
            Self::PropertyHistoryPostReply => Ok(Self::Recv::PropertyHistoryPostReply(
                DMGenericReply::s(&s, &caps)?,
            )),
            Self::ServicePropertySet => {
                let payload: AlinkRequest<Value> = serde_json::from_slice(payload)?;
                let data = PropertySet {
                    msg_id: payload.msg_id(),
                    params: payload.params,
                };
                let data = DataModelRecv::new(&caps[1], &caps[2], data);
                Ok(Self::Recv::ServicePropertySet(data))
            }
            Self::Service => {
                let payload: AlinkRequest<Value> = serde_json::from_slice(payload)?;
                let data = AsyncServiceInvoke {
                    msg_id: payload.msg_id(),
                    service_id: (&caps[3]).to_string(),
                    params: payload.params,
                };
                let data = DataModelRecv::new(&caps[1], &caps[2], data);
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
                let data = DataModelRecv::new(&caps[2], &caps[3], data);
                Ok(Self::Recv::RrpcService(data))
            }
            Self::ModelDownRaw => {
                let data = RawData {
                    data: payload.to_vec(),
                };
                let data = DataModelRecv::new(&caps[1], &caps[2], data);
                Ok(Self::Recv::ModelDownRaw(data))
            }
            Self::ModelUpRawReply => {
                let data = RawData {
                    data: payload.to_vec(),
                };
                let data = DataModelRecv::new(&caps[1], &caps[2], data);
                Ok(Self::Recv::ModelUpRawReply(data))
            }
            Self::RrpcDownRaw => {
                let data = RawServiceInvoke {
                    rrpc_id: (&caps[1]).to_string(),
                    data: payload.to_vec(),
                };
                let data = DataModelRecv::new(&caps[2], &caps[3], data);
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

pub type DMGenericReply = DataModelRecv<GenericReply>;

impl DMGenericReply {
    pub fn s(s: &str, caps: &Vec<String>) -> crate::Result<Self> {
        let payload: AlinkResponse<Value> = serde_json::from_str(&s)?;
        let data = GenericReply {
            msg_id: payload.msg_id(),
            code: payload.code,
            data: payload.data,
            message: payload.message.unwrap_or("".to_string()),
        };
        Ok(DataModelRecv::new(
            caps.get(1).ok_or_else(|| Error::DeviceNameUnmatched)?,
            caps.get(2).ok_or_else(|| Error::DeviceNameUnmatched)?,
            data,
        ))
    }
}
