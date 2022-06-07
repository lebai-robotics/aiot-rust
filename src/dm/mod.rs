//! 物模型

use crate::alink::aiot_module::{AiotModule, ModuleRecvKind};
use crate::alink::{AlinkRequest, AlinkResponse};
use crate::mqtt::MqttConnection;
use crate::{Error, Result, ThreeTuple};
use enum_iterator::IntoEnumIterator;
use log::{debug, info};
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
pub use base::*;

pub type Recv = RecvEnum;
pub type RecvKind = RecvEnumKind;
pub type Module = AiotModule<Recv, DataModelOptions>;

impl Module {
    pub async fn init(&self) -> Result<()> {
        let mut client = self.client.clone();
        let mut topics = rumqttc::Subscribe::empty_subscribe();
        for item in RecvKind::into_enum_iter() {
            let topic = item.get_topic();
            topics.add(topic.topic.to_string(), QoS::AtMostOnce);
        }
        client.subscribe_many(topics.filters).await?;
        Ok(())
    }
}

impl MqttConnection {
    pub fn data_model(&mut self, options: DataModelOptions) -> Result<Module> {
        let (tx, rx) = mpsc::channel(64);
        let executor = Executor {
            tx,
            three: self.mqtt_client.three.clone(),
        };
        self.module(Box::new(executor), rx, options)
    }
}

pub struct Executor {
    three: Arc<ThreeTuple>,
    tx: Sender<Recv>,
}

#[async_trait::async_trait]
impl crate::Executor for Executor {
    async fn execute(&self, topic: &str, payload: &[u8]) -> crate::Result<()> {
        let data = crate::execute::<RecvKind>(&self.three, topic, payload)?;
        self.tx.send(data).await.map_err(|_| Error::MpscSendError)
    }
}

/// 物模型设置
#[derive(Debug, Clone)]
pub struct DataModelOptions {
    /// 用户是否希望接收post消息后的reply
    pub post_reply: bool,
}

impl DataModelOptions {
    pub fn new() -> Self {
        Self { post_reply: true }
    }
}
