use crate::alink::aiot_module::AiotModule;
use crate::alink::aiot_module::ModuleRecvKind;
use crate::bootstrap::push::*;
use crate::bootstrap::recv::*;
use crate::mqtt::MqttConnection;
use crate::{Error, Result, ThreeTuple};
use enum_iterator::IntoEnumIterator;
use log::*;
use rumqttc::{AsyncClient, QoS};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

pub mod base;
pub mod push;
pub mod recv;

pub type Recv = BootstrapRecv;
pub type RecvKind = BootstrapRecvKind;
pub type Module = AiotModule<Recv>;

impl MqttConnection {
    pub fn bootstrap(&mut self) -> Result<Module> {
        let (tx, rx) = mpsc::channel(64);
        let executor = Executor {
            tx,
            three: self.mqtt_client.three.clone(),
        };

        self.module(Box::new(executor), rx, ())
    }
}

pub struct Executor {
    three: Arc<ThreeTuple>,
    tx: Sender<Recv>,
}

#[async_trait::async_trait]
impl crate::Executor for Executor {
    async fn execute(&mut self, topic: &str, payload: &[u8]) -> crate::Result<()> {
        let data = crate::execute::<RecvKind>(&self.three, topic, payload)?;
        self.tx.send(data).await.map_err(|_| Error::MpscSendError)
    }
}
