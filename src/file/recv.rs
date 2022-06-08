use super::base::*;
use crate::alink::aiot_module::{get_aiot_json, ModuleRecvKind};
use crate::alink::alink_topic::ALinkSubscribeTopic;
use crate::alink::{AlinkRequest, AlinkResponse, ParamsRequest, SimpleResponse};
use crate::{Error, Result, ThreeTuple};
use enum_iterator::IntoEnumIterator;
use enum_kinds::EnumKind;
use regex::Regex;
use rumqttc::{AsyncClient, QoS};
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

/// 设备请求上传文件
/// 响应Topic：`/sys/${productKey}/${deviceName}/thing/file/upload/mqtt/init_reply`。
pub type InitReply = AlinkResponse<InitData, Option<String>>;

/// 设备上传文件分片
/// 响应Topic：`/sys/${productKey}/${deviceName}/thing/file/upload/mqtt/send_reply`。
pub type SendReply = AlinkResponse<SendReplyData, Option<String>>;

/// 设备取消上传文件
/// 响应Topic：`/sys/${productKey}/${deviceName}/thing/file/upload/mqtt/cancel_reply`。
pub type CancelReply = AlinkResponse<UploadId, Option<String>>;

#[derive(Debug, EnumKind, Clone)]
#[enum_kind(FileRecvKind, derive(Serialize, IntoEnumIterator, Deserialize))]
pub enum FileRecv {
    InitReply(InitReply),
    SendReply(SendReply),
    CancelReply(CancelReply),
}

impl ModuleRecvKind for super::RecvKind {
    type Recv = super::Recv;

    fn to_payload(&self, payload: &[u8], _: &Vec<String>) -> crate::Result<FileRecv> {
        let s = get_aiot_json(payload);
        match *self {
            Self::InitReply => Ok(Self::Recv::InitReply(serde_json::from_str(&s)?)),
            Self::SendReply => Ok(Self::Recv::SendReply(serde_json::from_str(&s)?)),
            Self::CancelReply => Ok(Self::Recv::CancelReply(serde_json::from_str(&s)?)),
        }
    }

    fn get_topic(&self) -> ALinkSubscribeTopic {
        match *self {
            Self::InitReply => {
                ALinkSubscribeTopic::new("/sys/+/+/thing/file/upload/mqtt/init_reply")
            }
            Self::SendReply => {
                ALinkSubscribeTopic::new("/sys/+/+/thing/file/upload/mqtt/send_reply")
            }
            Self::CancelReply => {
                ALinkSubscribeTopic::new("/sys/+/+/thing/file/upload/mqtt/cancel_reply")
            }
        }
    }
}
