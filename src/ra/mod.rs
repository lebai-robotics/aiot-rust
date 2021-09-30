//! 远程登录

mod protocol;
mod proxy;
mod session;

use crate::{Error, Result, ThreeTuple};
use proxy::RemoteAccessProxy;
use rumqttc::{AsyncClient, QoS};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub struct RemoteAccessOptions {
    three: Arc<ThreeTuple>,
    cloud_host: String,
    //远程连接通道云端服务地址，可以是域名
    cloud_port: u32,    //远程连接通道云端服务端口
}

impl RemoteAccessOptions {
    pub fn new(three: Arc<ThreeTuple>) -> Self {
        Self {
            three,
            cloud_host: "backend-iotx-remote-debug.aliyun.com".to_string(),
            cloud_port: 443,
        }
    }

    pub fn switch_topic(&self) -> String {
        format!(
            "/sys/{}/{}/edge/debug/switch",
            self.three.product_key, self.three.device_name
        )
    }
}

pub struct Runner {
    topic: String,
    rap: RemoteAccessProxy,
}

impl Runner {
    pub async fn init(&self, client: &AsyncClient) -> Result<()> {
        client.subscribe(&self.topic, QoS::AtLeastOnce).await?;
        Ok(())
    }

    pub async fn poll(&mut self) -> Result<()> {
        self.rap.process().await
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

struct RemoteAccessInner {
    runner: Runner,
    executor: Executor,
}

impl RemoteAccessInner {
    fn new(three: Arc<ThreeTuple>) -> Result<Self> {
        let ra = RemoteAccessOptions::new(three);
        let topic = ra.switch_topic();
        let (tx, rx) = mpsc::channel(16);
        let rap = RemoteAccessProxy::new(rx, ra)?;
        let executor = Executor {
            topic: topic.clone(),
            tx,
        };
        let runner = Runner { topic, rap };
        Ok(Self { runner, executor })
    }
}

pub trait RemoteAccess {
    fn remote_access(&mut self) -> Result<Runner>;
}

impl RemoteAccess for crate::MqttClient {
    fn remote_access(&mut self) -> Result<Runner> {
        let ra = RemoteAccessInner::new(self.three.clone())?;
        self.executors
            .push(Box::new(ra.executor) as Box<dyn crate::Executor>);
        Ok(ra.runner)
    }
}
