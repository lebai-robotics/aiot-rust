use crate::{Error, Result, ThreeTuple};
use log::*;
use rumqttc::{AsyncClient, QoS};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NtpRequest {
    #[serde(rename = "deviceSendTime")]
    pub device_send_time: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NtpResponse {
    #[serde(rename = "deviceSendTime")]
    pub device_send_time: i64,
    #[serde(rename = "serverRecvTime")]
    pub server_recv_time: i64,
    #[serde(rename = "serverSendTime")]
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

pub struct HalfRunner {
    topic: String,
    rx: Receiver<Vec<u8>>,
}

impl HalfRunner {
    pub async fn init(self, client: &AsyncClient) -> Result<Runner> {
        client.subscribe(&self.topic, QoS::AtLeastOnce).await?;
        Ok(Runner {
            topic: self.topic,
            rx: self.rx,
            client: client.clone(),
        })
    }
}

pub struct Runner {
    topic: String,
    rx: Receiver<Vec<u8>>,
    client: AsyncClient,
}

impl Runner {
    pub async fn send(&self) -> Result<()> {
        use std::time::SystemTime;
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
        let data = NtpRequest {
            device_send_time: now.as_millis() as u64,
        };
        let data = serde_json::to_string(&data)?;
        self.client
            .publish(&self.topic, QoS::AtMostOnce, false, data)
            .await?;
        Ok(())
    }

    pub async fn poll(&mut self) -> Result<NtpResponse> {
        let res = self.rx.recv().await.ok_or(Error::ReceiveCloudError)?;
        let data: NtpResponse = serde_json::from_slice(&res)?;
        debug!("{:?}", data);
        Ok(data)
    }
}

pub struct Executor {
    topic: String,
    tx: Sender<Vec<u8>>,
}

#[async_trait::async_trait]
impl crate::Executor for Executor {
    async fn execute(&self, topic: &str, payload: &[u8]) -> Result<()> {
        if topic == &self.topic {
            self.tx
                .send(payload.to_vec())
                .await
                .map_err(|_| Error::SendTopicError)?;
        }
        Ok(())
    }
}

pub struct NtpService {
    pub runner: HalfRunner,
    pub executor: Executor,
}

impl NtpService {
    pub fn new(three: Arc<ThreeTuple>) -> Result<Self> {
        let (tx, rx) = mpsc::channel(16);
        let topic = format!(
            "/ext/ntp/{}/{}/response",
            three.product_key, three.device_name
        );
        let executor = Executor { topic, tx };
        let topic = format!(
            "/ext/ntp/{}/{}/request",
            three.product_key, three.device_name
        );
        let runner = HalfRunner { topic, rx };
        Ok(Self { runner, executor })
    }
}

pub trait NtpServiceTrait {
    fn ntp_service(&mut self) -> Result<HalfRunner>;
}

impl NtpServiceTrait for crate::MqttClient {
    fn ntp_service(&mut self) -> Result<HalfRunner> {
        let ra = NtpService::new(self.three.clone())?;
        self.executors
            .push(Box::new(ra.executor) as Box<dyn crate::Executor>);
        Ok(ra.runner)
    }
}
