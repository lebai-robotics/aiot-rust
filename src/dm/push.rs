use super::base::*;
use crate::alink::{AlinkRequest, AlinkResponse, SysAck};
use crate::{Error, Result, ThreeTuple};
use regex::Regex;
use rumqttc::{AsyncClient, QoS};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

impl super::Module {
    pub async fn send(&self, data: DataModelMsg) -> crate::Result<()> {
        let mut data = data;
        if data.product_key.is_none() {
            data.product_key = Some(self.three.product_key.to_string());
        }
        if data.device_name.is_none() {
            data.device_name = Some(self.three.device_name.to_string());
        }
        let (topic, payload) = data.to_payload(1)?; // TODO: ack
        self.publish(topic, &payload).await
    }
}
