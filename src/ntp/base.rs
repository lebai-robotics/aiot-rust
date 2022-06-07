use crate::alink::{AlinkRequest, AlinkResponse};
use crate::{Error, Result, ThreeTuple};
use log::*;
use regex::Regex;
use rumqttc::{AsyncClient, QoS};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NtpRequest {
    pub device_send_time: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NtpResponse {
    pub device_send_time: i64,
    pub server_recv_time: i64,
    pub server_send_time: i64,
}

impl NtpResponse {
    /// 设备端计算出服务端当前精确的Unix时间，并设置为当前系统时间。
    /// 设备端收到服务端的时间记为${deviceRecvTime}，则设备上的精确时间为：(${serverRecvTime}+${serverSendTime}+${deviceRecvTime}-${deviceSendTime})/2。
    pub async fn calc(&self) -> Result<chrono::NaiveDateTime> {
        use chrono::{NaiveDateTime, Utc};
        let now = Utc::now().timestamp_millis();
        let utc = (self.server_recv_time + self.server_send_time + now - self.device_send_time) / 2;
        let utc = NaiveDateTime::from_timestamp(utc / 1000, (utc % 1000) as u32 * 1_000_000);
        debug!("{}", utc.to_string());
        // TODO: 设置系统时间
        Ok(utc)
    }
}
