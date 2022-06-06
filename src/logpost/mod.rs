//! 日志上报。

use crate::alink::aiot_module::AiotModule;
use crate::alink::aiot_module::ModuleRecvKind;
use crate::alink::{AlinkRequest, AlinkResponse};
use crate::mqtt::MqttConnection;
use crate::{Error, Result, ThreeTuple};
use enum_iterator::IntoEnumIterator;
use log::*;
use regex::Regex;
use rumqttc::{AsyncClient, QoS};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use self::recv::*;

pub mod base;
pub mod push;
pub mod recv;

pub type Recv = LogPostRecv;
pub type RecvKind = LogPostRecvKind;
pub type Module = AiotModule<Recv>;

impl MqttConnection {
    pub fn log_post(&mut self) -> Result<Module> {
        let (tx, rx) = mpsc::channel(64);
        let executor = Executor {
            tx,
            three: self.mqtt_client.three.clone(),
        };
        self.module(Box::new(executor), rx)
    }
}

pub struct Executor {
    three: Arc<ThreeTuple>,
    tx: Sender<Recv>,
}

#[async_trait::async_trait]
impl crate::Executor for Executor {
    async fn execute(&self, topic: &str, payload: &[u8]) -> crate::Result<()> {
        debug!("receive: {} {}", topic, String::from_utf8_lossy(payload));
        if let Some(kind) =
            RecvKind::match_kind(topic, &self.three.product_key, &self.three.device_name)
        {
            let data = kind.to_payload(payload)?;
            self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
        } else {
            debug!("no match topic: {}", topic);
        }
        Ok(())
    }
}