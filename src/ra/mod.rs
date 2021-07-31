mod protocol;
mod proxy;
mod session;

use crate::{Error, Result, ThreeTuple};
use proxy::RemoteAccessProxy;
use rumqttc::{AsyncClient, QoS};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub struct RemoteAccessOptions {
    pk: String,         //云端PK
    dn: String,         //云端DN
    ds: String,         //云端DS
    cloud_host: String, //远程连接通道云端服务地址，可以是域名
    cloud_port: u32,    //远程连接通道云端服务端口
}

impl RemoteAccessOptions {
    pub fn new(three: &ThreeTuple) -> Self {
        Self {
            pk: three.product_key.to_string(),
            dn: three.device_name.to_string(),
            ds: three.device_secret.to_string(),
            cloud_host: "backend-iotx-remote-debug.aliyun.com".to_string(),
            cloud_port: 443,
        }
    }

    pub fn switch_topic(&self) -> String {
        format!("/sys/{}/{}/edge/debug/switch", self.pk, self.dn)
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

pub struct RemoteAccess {
    pub runner: Runner,
    pub executor: Executor,
}

impl RemoteAccess {
    pub fn new(three: &ThreeTuple) -> Result<Self> {
        let ra = RemoteAccessOptions::new(&three);
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

pub trait RemoteAccessTrait {
    fn remote_access(&mut self, three: &ThreeTuple) -> Result<Runner>;
}

impl RemoteAccessTrait for crate::MqttClient {
    fn remote_access(&mut self, three: &ThreeTuple) -> Result<Runner> {
        let ra = RemoteAccess::new(&three)?;
        self.executors
            .push(Box::new(ra.executor) as Box<dyn crate::Executor>);
        Ok(ra.runner)
    }
}
