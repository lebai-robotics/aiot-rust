use super::base::{EdgeDebugSwitch, SecureTunnelNotify};
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

// /sys/${productKey}/${deviceName}/edge/debug/switch
pub type DebugSwitch = EdgeDebugSwitch;

// /sys/%s/%s/secure_tunnel/notify
pub type Switch = AlinkResponse<SecureTunnelNotify>;

// /sys/%s/%s/secure_tunnel/proxy/request_reply
pub type RequestReply = AlinkResponse<SecureTunnelNotify>;

#[derive(Debug, EnumKind, Clone)]
#[enum_kind(RemoteAccessRecvKind, derive(Serialize, IntoEnumIterator, Deserialize))]
pub enum RemoteAccessRecv {
    // DebugSwitch(DebugSwitch),
    Switch(Switch),
    RequestReply(RequestReply),
}

impl RemoteAccessRecv {
    pub fn is_open(&self) -> bool {
        match self {
            // RemoteAccessRecv::DebugSwitch(data) => data.is_open(),
            _ => false,
        }
    }
}

impl ModuleRecvKind for super::RecvKind {
    type Recv = super::Recv;

    fn to_payload(&self, payload: &[u8], _: &Vec<String>) -> crate::Result<RemoteAccessRecv> {
        let s = get_aiot_json(payload);
        match *self {
            // Self::DebugSwitch => Ok(Self::Recv::DebugSwitch(serde_json::from_str(&s)?)),
            Self::Switch => Ok(Self::Recv::Switch(serde_json::from_str(&s)?)),
            Self::RequestReply => Ok(Self::Recv::RequestReply(serde_json::from_str(&s)?)),
        }
    }

    fn get_topic(&self) -> ALinkSubscribeTopic {
        match *self {
            // Self::DebugSwitch => ALinkSubscribeTopic::new("/sys/+/+/edge/debug/switch"),
            Self::Switch => ALinkSubscribeTopic::new("/sys/+/+/secure_tunnel/notify"),
            Self::RequestReply => {
                ALinkSubscribeTopic::new("/sys/+/+/secure_tunnel/proxy/request_reply")
            }
        }
    }
}
