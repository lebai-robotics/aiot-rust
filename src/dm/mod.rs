pub mod msg;
pub mod recv;

use crate::alink::*;
use crate::{Error, Result, ThreeTuple};
use rumqttc::{AsyncClient, QoS};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use log::*;

#[derive(Debug, Clone)]
pub struct DataModelOptions {
    // 用户是否希望接收post消息后的reply
    pub post_reply: bool,
}

impl DataModelOptions {
    pub fn new() -> Self {
        Self { post_reply: true }
    }
}

pub struct HalfRunner {
    rx: Receiver<recv::RecvEnum>,
    post_reply: bool,
    three: ThreeTuple,
}

impl HalfRunner {
    pub async fn init(self, mut client: AsyncClient) -> Result<Runner> {
        let mut topics = rumqttc::Subscribe::empty_subscribe();
        for &topic in TOPICS {
            topics.add(topic.to_string(), QoS::AtMostOnce);
        }
        client.subscribe_many(topics.filters).await?;
        Ok(Runner {
            rx: self.rx,
            ack: if self.post_reply { 1 } else { 0 },
            client,
            three: self.three,
        })
    }
}

pub struct Runner {
    rx: Receiver<recv::RecvEnum>,
    client: AsyncClient,
    ack: i32,
    three: ThreeTuple,
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
        let topic = data.topic();
        debug!("topic={}", topic);
        let method = data.data.method();
        let value = data.data.value();
        let payload = AlinkRequest::new(method, value, self.ack);
        let payload = serde_json::to_string(&payload)?;
        debug!("payload={}", payload);
        self.client
            .publish(topic, QoS::AtMostOnce, false, payload)
            .await?;
        Ok(())
    }

    pub async fn poll(&mut self) -> Result<()> {
        // self.rap.process().await
        Ok(())
    }
}

pub struct Executor {
    tx: Sender<recv::RecvEnum>,
}

#[async_trait::async_trait]
impl crate::Executor for Executor {
    async fn execute(&self, topic: &str, payload: &[u8]) -> Result<()> {
        // if topic == &self.topic {
        //     // self.tx
        //     //     .send(payload.to_vec())
        //     //     .await
        //     //     .map_err(|_| Error::SendTopicError)?;
        // }
        Ok(())
    }
}

pub struct DataModel {
    runner: HalfRunner,
    executor: Executor,
}

impl DataModel {
    pub fn new(options: &DataModelOptions, three: &ThreeTuple) -> Result<Self> {
        let (tx, rx) = mpsc::channel(64);
        let executor = Executor { tx };
        let runner = HalfRunner {
            rx,
            post_reply: options.post_reply,
            three: three.clone(),
        };
        Ok(Self { runner, executor })
    }
}

pub trait DataModelTrait {
    fn data_model(&mut self, options: &DataModelOptions) -> Result<HalfRunner>;
}

impl DataModelTrait for crate::MqttClient {
    fn data_model(&mut self, options: &DataModelOptions) -> Result<HalfRunner> {
        let ra = DataModel::new(&options, &self.three)?;
        self.executors
            .push(Box::new(ra.executor) as Box<dyn crate::Executor>);
        Ok(ra.runner)
    }
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
];
