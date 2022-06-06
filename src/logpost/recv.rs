use super::base::{LogConfig, LogItem};
use crate::alink::aiot_module::{get_aiot_json, ModuleRecvKind};
use crate::alink::alink_topic::ALinkSubscribeTopic;
use crate::alink::{AlinkRequest, AlinkResponse, SimpleResponse};
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

// 设备获取日志配置
// /sys/${productKey}/${deviceName}/thing/config/log/get_reply
pub type ConfigLogGetReply = AlinkResponse<LogConfig>;

// 设备接收订阅云端推送日志配置
// /sys/${productKey}/${deviceName}/thing/config/log/push
pub type ConfigLogPush = AlinkRequest<LogConfig>;

// 设备上报日志内容响应
// /sys/${productKey}/${deviceName}/thing/log/post_reply
pub type LogPostReply = SimpleResponse;

#[derive(Debug, EnumKind)]
#[enum_kind(LogPostRecvKind, derive(Serialize, IntoEnumIterator, Deserialize))]
pub enum LogPostRecv {
    /// 设备获取日志配置
    ConfigLogGetReply(ConfigLogGetReply),
    /// 设备接收订阅云端推送日志配置
    ConfigLogPush(ConfigLogPush),
    /// 设备上报日志内容响应
    LogPostReply(LogPostReply),
}

impl ModuleRecvKind for super::RecvKind {
    type Recv = super::Recv;

    fn to_payload(&self, payload: &[u8]) -> crate::Result<LogPostRecv> {
        let s = get_aiot_json(payload);
        match *self {
            Self::ConfigLogGetReply => Ok(Self::Recv::ConfigLogGetReply(serde_json::from_str(&s)?)),
            Self::ConfigLogPush => Ok(Self::Recv::ConfigLogPush(serde_json::from_str(&s)?)),
            Self::LogPostReply => Ok(Self::Recv::LogPostReply(serde_json::from_str(&s)?)),
        }
    }

    fn get_topic(&self) -> ALinkSubscribeTopic {
        match *self {
            Self::ConfigLogGetReply => {
                ALinkSubscribeTopic::new("/sys/+/+/thing/config/log/get_reply")
            }
            Self::ConfigLogPush => ALinkSubscribeTopic::new("/sys/+/+/thing/config/log/push"),
            Self::LogPostReply => ALinkSubscribeTopic::new("/sys/+/+/thing/log/post_reply"),
        }
    }
}
