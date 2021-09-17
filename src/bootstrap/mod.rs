use crate::alink::aiot_module::AiotModule;
use crate::alink::aiot_module::BaseExecutor;
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
pub mod push;
pub mod recv;

pub type BootstrapModule = AiotModule<BootstrapRecv>;

impl MqttConnection {
   pub fn remote_config(&mut self) -> Result<BootstrapModule> {
      let (tx, rx) = mpsc::channel(64);
      let executor = BaseExecutor::<BootstrapRecv>::new(tx, self.mqtt_client.three.clone());

      self.module(Box::new(executor), rx)
   }
}
