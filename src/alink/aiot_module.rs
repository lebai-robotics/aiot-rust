use crate::Error;
use crate::{mqtt::MqttConnection, Result};
use enum_iterator::IntoEnumIterator;
use enum_kinds::EnumKind;
use lazy_static::__Deref;
use log::debug;
use rumqttc::{AsyncClient, QoS};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::ThreeTuple;
use tokio::sync::mpsc;

use super::alink_topic::ALinkSubscribeTopic;

pub trait ModuleRecvKind: IntoEnumIterator {
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
   fn to_payload<TRecv>(&self, payload: &[u8]) -> Arc<TRecv>;
   fn get_topic(&self) -> ALinkSubscribeTopic;
}

pub trait ModuleRecv
where
   Self::Kind: ModuleRecvKind,
{
   type Kind;

   fn new(kind: Self::Kind, payload: &[u8]) -> Self;
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
      // let (tx, rx) = mpsc::channel(64);
      // let executor = Executor::<TModuleRecv>::new {
      //    tx,
      //    three: self.mqtt_client.three.clone(),
      // };

      self.mqtt_client.executors.push(executor);
      let runner = AiotModule::<TModuleRecv> {
         rx,
         three: self.mqtt_client.three.clone(),
         client: self.mqtt.clone(),
      };
      Ok(runner)
   }
}

impl<TModuleRecv> BaseExecutor<TModuleRecv>
where
   TModuleRecv: ModuleRecv,
{
   fn get_recv(&self, topic: &str, payload: &[u8]) -> Option<TModuleRecv> {
      debug!("receive: {} {}", topic, String::from_utf8_lossy(payload));
      if let Some(kind) =
         TModuleRecv::Kind::match_kind(topic, &self.three.product_key, &self.three.device_name)
      {
         let data = TModuleRecv::new(kind, payload);
         return Some(data);
         // self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
      } else {
         debug!("no match topic: {}", topic);
      }
      None
   }
}

impl<TRecv> AiotModule<TRecv> {
   pub async fn publish<T>(&self, topic: String, payload: &T) -> Result<()>
   where
      T: ?Sized + Serialize,
   {
      let payload = serde_json::to_vec(payload)?;
      debug!("publish: {} {}", topic, String::from_utf8_lossy(&payload));
      self
         .client
         .publish(topic, QoS::AtMostOnce, false, payload)
         .await?;
      Ok(())
   }
}

pub struct BaseExecutor<TRecv> {
   pub three: Arc<ThreeTuple>,
   pub tx: Sender<TRecv>,
}

impl<TRecv> BaseExecutor<TRecv> {
   pub fn new(tx: Sender<TRecv>, three: Arc<ThreeTuple>) -> Self {
      BaseExecutor::<TRecv> { three, tx }
   }
}
