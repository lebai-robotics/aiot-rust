use super::alink_topic::ALinkSubscribeTopic;
use crate::Error;
use crate::ThreeTuple;
use crate::{mqtt::MqttConnection, Result};
use enum_iterator::IntoEnumIterator;
use enum_kinds::EnumKind;
use lazy_static::__Deref;
use log::debug;
use rumqttc::{AsyncClient, QoS};
use serde::Serialize;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

pub trait ModuleRecvKind: IntoEnumIterator {
    type Recv;
    fn match_kind(topic: &str, product_key: &str, device_name: &str) -> Option<Self> {
        for item in Self::into_enum_iter() {
            let alink_topic = item.get_topic();
            if !alink_topic.is_match(topic, product_key, device_name) {
                continue;
            }
            return Some(item);
        }
        None
    }
    fn to_payload(&self, payload: &[u8]) -> Result<Self::Recv>;
    fn get_topic(&self) -> ALinkSubscribeTopic;
}

pub struct AiotModule<TRecv> {
    pub rx: Receiver<TRecv>,
    pub client: Arc<AsyncClient>,
    pub three: Arc<ThreeTuple>,
}

impl MqttConnection {
    pub fn module<TModuleRecv>(
        &mut self,
        executor: Box<dyn crate::Executor>,
        rx: Receiver<TModuleRecv>,
    ) -> Result<AiotModule<TModuleRecv>> {
        self.mqtt_client.executors.push(executor);
        let runner = AiotModule::<TModuleRecv> {
            rx,
            three: self.mqtt_client.three.clone(),
            client: self.mqtt.clone(),
        };
        Ok(runner)
    }
}

impl<TRecv> AiotModule<TRecv> {
    pub async fn publish<T>(&self, topic: String, payload: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let payload = serde_json::to_vec(payload)?;
        debug!("publish: {} {}", topic, String::from_utf8_lossy(&payload));
        self.client
            .publish(topic, QoS::AtMostOnce, false, payload)
            .await?;
        Ok(())
    }

    pub async fn poll(&mut self) -> Result<TRecv> {
        self.rx.recv().await.ok_or(Error::RecvTopicError)
    }
}

pub fn get_aiot_json(payload: &[u8]) -> String {
    String::from_utf8_lossy(payload).replace(",\"data\":{},", ",\"data\":null,")
}
