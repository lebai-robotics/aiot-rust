//! 远程登录

use crate::alink::aiot_module::{AiotModule, ModuleRecvKind};
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

use self::base::RemoteAccessOptions;
use self::recv::*;

pub mod base;
pub mod push;
pub mod recv;

mod protocol;
mod proxy;
mod session;
use proxy::RemoteAccessProxy;

pub type Recv = RemoteAccessRecv;
pub type RecvKind = RemoteAccessRecvKind;
pub type Module = AiotModule<Recv>;

impl Module {
    pub async fn init(&self) -> Result<()> {
        for item in RecvKind::into_enum_iter() {
            let topic = item.get_topic();
            self.client.subscribe(topic.topic, QoS::AtLeastOnce).await?;
        }
        Ok(())
    }
}

impl MqttConnection {
    pub fn remote_access(&mut self) -> Result<(Module, RemoteAccessProxy)> {
        let (tx, rx) = mpsc::channel(16);
        let (tx_, rx_) = mpsc::channel(16);
        let ra = RemoteAccessOptions::new(self.mqtt_client.three.clone());
        let rap = RemoteAccessProxy::new(rx_, ra)?;
        let executor = Executor {
            tx,
            tx_,
            three: self.mqtt_client.three.clone(),
        };
        let module = self.module(Box::new(executor), rx)?;
        Ok((module, rap))
    }
}

pub struct Executor {
    three: Arc<ThreeTuple>,
    tx: Sender<Recv>,
    tx_: Sender<Recv>,
}

#[async_trait::async_trait]
impl crate::Executor for Executor {
    async fn execute(&self, topic: &str, payload: &[u8]) -> crate::Result<()> {
        debug!("receive: {} {}", topic, String::from_utf8_lossy(payload));
        if let Some(kind) =
            RecvKind::match_kind(topic, &self.three.product_key, &self.three.device_name)
        {
            let data = kind.to_payload(payload)?;
            self.tx
                .send(data.clone())
                .await
                .map_err(|_| Error::MpscSendError)?;
            self.tx_
                .send(data)
                .await
                .map_err(|_| Error::MpscSendError)?;
        } else {
            debug!("no match topic: {}", topic);
        }
        Ok(())
    }
}
