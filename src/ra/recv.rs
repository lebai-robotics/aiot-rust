use super::base::{ConnectOrUpdate, EdgeDebugSwitch, SecureTunnelNotify};
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
pub type Switch = SecureTunnelNotify;

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
    pub fn into_inner(self) -> SecureTunnelNotify {
        match self {
            RemoteAccessRecv::Switch(data) => data,
            RemoteAccessRecv::RequestReply(data) => data.data,
        }
    }
}

impl ModuleRecvKind for super::RecvKind {
    type Recv = super::Recv;

    fn to_payload(&self, payload: &[u8], _: &Vec<String>) -> crate::Result<RemoteAccessRecv> {
        let s = get_aiot_json(payload);
        match *self {
            // Self::DebugSwitch => Ok(Self::Recv::DebugSwitch(serde_json::from_str(&s)?)),
            Self::Switch => {
                let data: ConnectOrUpdate = serde_json::from_str(&s)?;
                Ok(Self::Recv::Switch(data.into()))
            }
            Self::RequestReply => {
                let data: AlinkResponse<ConnectOrUpdate> = serde_json::from_str(&s)?;
                Ok(Self::Recv::RequestReply(AlinkResponse {
                    id: data.id,
                    code: data.code,
                    data: data.data.into(),
                    message: None,
                    version: None,
                    method: None,
                }))
            }
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
