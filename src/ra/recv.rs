use super::base::EdgeDebugSwitch;
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
pub type EdgeDebugSwitchRequest = EdgeDebugSwitch;

#[derive(Debug, EnumKind, Clone)]
#[enum_kind(RemoteAccessRecvKind, derive(Serialize, IntoEnumIterator, Deserialize))]
pub enum RemoteAccessRecv {
    EdgeDebugSwitchRequest(EdgeDebugSwitchRequest),
}

impl RemoteAccessRecv {
    pub fn is_open(&self) -> bool {
        match self {
            RemoteAccessRecv::EdgeDebugSwitchRequest(data) => data.is_open(),
        }
    }
}

impl ModuleRecvKind for super::RecvKind {
    type Recv = super::Recv;

    fn to_payload(&self, payload: &[u8]) -> crate::Result<RemoteAccessRecv> {
        let s = get_aiot_json(payload);
        match *self {
            Self::EdgeDebugSwitchRequest => Ok(Self::Recv::EdgeDebugSwitchRequest(
                serde_json::from_str(&s)?,
            )),
        }
    }

    fn get_topic(&self) -> ALinkSubscribeTopic {
        match *self {
            Self::EdgeDebugSwitchRequest => ALinkSubscribeTopic::new("/sys/+/+/edge/debug/switch"),
        }
    }
}
