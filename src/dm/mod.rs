//! 物模型

pub mod msg;
pub mod recv;

use crate::alink::*;
use crate::{Error, Result, ThreeTuple};
use log::*;
use regex::Regex;
use rumqttc::{AsyncClient, QoS};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

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

pub struct HalfRunner {
    rx: Receiver<recv::DataModelRecv>,
    post_reply: bool,
    three: Arc<ThreeTuple>,
}

impl HalfRunner {
    pub async fn init(self, client: &AsyncClient) -> Result<Runner> {
        let mut client = client.clone();
        let mut topics = rumqttc::Subscribe::empty_subscribe();
        for &topic in TOPICS {
            topics.add(topic.to_string(), QoS::AtMostOnce);
        }
        client.subscribe_many(topics.filters).await?;
        Ok(Runner {
            rx: self.rx,
            ack: if self.post_reply { 1 } else { 0 },
            client,
            three: self.three.clone(),
        })
    }
}

pub struct Runner {
    rx: Receiver<recv::DataModelRecv>,
    client: AsyncClient,
    ack: i32,
    three: Arc<ThreeTuple>,
}

impl Runner {
    pub async fn send(&mut self, data: msg::DataModelMsg) -> Result<()> {
        let mut data = data;
        if data.product_key.is_none() {
            data.product_key = Some(self.three.product_key.to_string());
        }
        if data.device_name.is_none() {
            data.device_name = Some(self.three.device_name.to_string());
        }
        let (topic, payload) = data.to_payload(self.ack)?;
        debug!("payload={}", String::from_utf8_lossy(&payload));
        self.client
            .publish(topic, QoS::AtMostOnce, false, payload)
            .await?;
        Ok(())
    }

    pub async fn poll(&mut self) -> Result<recv::DataModelRecv> {
        self.rx.recv().await.ok_or(Error::RecvTopicError)
    }
}

pub struct Executor {
    three: Arc<ThreeTuple>,
    tx: Sender<recv::DataModelRecv>,
    regs: [Regex; 11],
}

#[async_trait::async_trait]
impl crate::Executor for Executor {
    async fn execute(&self, topic: &str, payload: &[u8]) -> Result<()> {
        debug!("receive: {} {}", topic, String::from_utf8_lossy(payload));

        for i in [0, 7, 8, 9, 10] {
            if let Some(caps) = self.regs[i].captures(topic) {
                if &caps[1] != self.three.product_key || &caps[2] != self.three.device_name {
                    return Ok(());
                }
                let payload: AlinkResponse<Value> = serde_json::from_slice(&payload)?;
                
                let data = recv::GenericReply {
                    msg_id: payload.msg_id(),
                    code: payload.code,
                    data: payload.data.clone(),
                    message: payload.message.unwrap_or("".to_string()),
                };
                let data = recv::DataModelRecv::generic_reply(&caps[1], &caps[2], data);
                self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
                return Ok(());
            }
        }
        // "/sys/+/+/thing/service/property/set"
        if let Some(caps) = self.regs[1].captures(topic) {
            if &caps[1] != self.three.product_key || &caps[2] != self.three.device_name {
                return Ok(());
            }
            let payload: AlinkRequest<Value> = serde_json::from_slice(&payload)?;
            let data = recv::PropertySet {
                msg_id: payload.msg_id(),
                params: payload.params.clone(),
            };
            let data = recv::DataModelRecv::property_set(&caps[1], &caps[2], data);
            self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
            return Ok(());
        }
        // "/sys/+/+/thing/service/+"
        if let Some(caps) = self.regs[2].captures(topic) {
            if &caps[1] != self.three.product_key || &caps[2] != self.three.device_name {
                return Ok(());
            }
            let payload: AlinkRequest<Value> = serde_json::from_slice(&payload)?;
            let data = recv::AsyncServiceInvoke {
                msg_id: payload.msg_id(),
                service_id: (&caps[3]).to_string(),
                params: payload.params.clone(),
            };
            let data = recv::DataModelRecv::async_service_invoke(&caps[1], &caps[2], data);
            self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
            return Ok(());
        }
        // "/ext/rrpc/+/sys/+/+/thing/service/+"
        if let Some(caps) = self.regs[3].captures(topic) {
            if &caps[2] != self.three.product_key || &caps[3] != self.three.device_name {
                return Ok(());
            }
            let payload: AlinkRequest<Value> = serde_json::from_slice(&payload)?;
            let data = recv::SyncServiceInvoke {
                rrpc_id: (&caps[1]).to_string(),
                msg_id: payload.msg_id(),
                service_id: (&caps[4]).to_string(),
                params: payload.params.clone(),
            };
            let data = recv::DataModelRecv::sync_service_invoke(&caps[2], &caps[3], data);
            self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
            return Ok(());
        }
        // "/sys/+/+/thing/model/down_raw"
        if let Some(caps) = self.regs[4].captures(topic) {
            if &caps[1] != self.three.product_key || &caps[2] != self.three.device_name {
                return Ok(());
            }
            let data = recv::RawData {
                data: payload.to_vec(),
            };
            let data = recv::DataModelRecv::raw_data(&caps[1], &caps[2], data);
            self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
            return Ok(());
        }
        // "/sys/+/+/thing/model/up_raw_reply"
        if let Some(caps) = self.regs[5].captures(topic) {
            if &caps[1] != self.three.product_key || &caps[2] != self.three.device_name {
                return Ok(());
            }
            let data = recv::RawData {
                data: payload.to_vec(),
            };
            let data = recv::DataModelRecv::raw_data_reply(&caps[1], &caps[2], data);
            self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
            return Ok(());
        }
        // "/ext/rrpc/+/sys/+/+/thing/model/down_raw"
        if let Some(caps) = self.regs[6].captures(topic) {
            if &caps[2] != self.three.product_key || &caps[3] != self.three.device_name {
                return Ok(());
            }
            let data = recv::RawServiceInvoke {
                rrpc_id: (&caps[1]).to_string(),
                data: payload.to_vec(),
            };
            let data = recv::DataModelRecv::raw_sync_service_invoke(&caps[2], &caps[3], data);
            self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
            return Ok(());
        }

        Ok(())
    }
}

struct DataModelInner {
    runner: HalfRunner,
    executor: Executor,
}

const TOPICS: &'static [&str] = &[
    "/sys/+/+/thing/event/+/post_reply",
    "/sys/+/+/thing/service/property/set",
    "/sys/+/+/thing/service/+",
    "/ext/rrpc/+/sys/+/+/thing/service/+",
    "/sys/+/+/thing/model/down_raw",
    "/sys/+/+/thing/model/up_raw_reply",
    "/ext/rrpc/+/sys/+/+/thing/model/down_raw",
    "/sys/+/+/thing/property/desired/get_reply",
    "/sys/+/+/thing/property/desired/delete_reply",
    "/sys/+/+/thing/event/property/batch/post_reply",
    "/sys/+/+/thing/event/property/history/post_reply",
];

impl DataModelInner {
    fn new(options: &DataModelOptions, three: Arc<ThreeTuple>) -> Result<Self> {
        let regs = [
            Regex::new(r"/sys/(.*)/(.*)/thing/event/(.*)/post_reply")?,
            Regex::new(r"/sys/(.*)/(.*)/thing/service/property/set")?,
            Regex::new(r"/sys/(.*)/(.*)/thing/service/(.*)")?,
            Regex::new(r"/ext/rrpc/(.*)/sys/(.*)/(.*)/thing/service/(.*)")?,
            Regex::new(r"/sys/(.*)/(.*)/thing/model/down_raw")?,
            Regex::new(r"/sys/(.*)/(.*)/thing/model/up_raw_reply")?,
            Regex::new(r"/ext/rrpc/(.*)/sys/(.*)/(.*)/thing/model/down_raw")?,
            Regex::new(r"/sys/(.*)/(.*)/thing/property/desired/get_reply")?,
            Regex::new(r"/sys/(.*)/(.*)/thing/property/desired/delete_reply")?,
            Regex::new(r"/sys/(.*)/(.*)/thing/event/property/batch/post_reply")?,
            Regex::new(r"/sys/(.*)/(.*)/thing/event/property/history/post_reply")?,
        ];
        let (tx, rx) = mpsc::channel(64);
        let runner = HalfRunner {
            rx,
            post_reply: options.post_reply,
            three: three.clone(),
        };
        let executor = Executor { tx, three, regs };
        Ok(Self { runner, executor })
    }
}

pub trait DataModel {
    fn data_model(&mut self, options: &DataModelOptions) -> Result<HalfRunner>;
}

impl DataModel for crate::MqttClient {
    fn data_model(&mut self, options: &DataModelOptions) -> Result<HalfRunner> {
        let ra = DataModelInner::new(&options, self.three.clone())?;
        self.executors
            .push(Box::new(ra.executor) as Box<dyn crate::Executor>);
        Ok(ra.runner)
    }
}
