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
    /// 上报设备时间
    pub async fn send(&self) -> crate::Result<()> {
        use std::time::SystemTime;
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
        let payload = NtpRequest {
            device_send_time: now.as_millis() as u64,
        };
        let topic = format!(
            "/ext/ntp/{}/{}/request",
            self.three.product_key, self.three.device_name
        );
        self.publish(topic, &payload).await
    }
}
