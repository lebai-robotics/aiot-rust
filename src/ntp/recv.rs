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
use std::any::TypeId;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

// /ext/ntp/${YourProductKey}/${YourDeviceName}/response
pub type NtpResponseType = NtpResponse;

#[derive(Debug, EnumKind)]
#[enum_kind(NtpRecvKind, derive(Serialize, IntoEnumIterator, Deserialize))]
pub enum NtpRecv {
    NtpResponseType(NtpResponseType),
}

impl NtpRecv {
    pub async fn calc(&self) -> Result<chrono::NaiveDateTime> {
        match self {
            Self::NtpResponseType(data) => data.calc().await,
        }
    }
}

impl ModuleRecvKind for super::RecvKind {
    type Recv = super::Recv;

    fn to_payload(&self, payload: &[u8], _: &Vec<String>) -> crate::Result<NtpRecv> {
        let s = get_aiot_json(payload);
        match *self {
            Self::NtpResponseType => Ok(Self::Recv::NtpResponseType(serde_json::from_str(&s)?)),
        }
    }

    fn get_topic(&self) -> ALinkSubscribeTopic {
        match *self {
            Self::NtpResponseType => ALinkSubscribeTopic::new("/ext/ntp/+/+/response"),
        }
    }
}
