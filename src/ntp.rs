use crate::{Result, Error, ThreeTuple};
use rumqttc::{AsyncClient, QoS};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};


pub struct Runner {
    topic: String,
    rap: NtpServiceProxy,
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

pub struct NtpService {
    pub runner: Runner,
    pub executor: Executor,
}

impl NtpService {
    pub fn new(three: &ThreeTuple) -> Result<Self> {
        let ra = NtpServiceOptions::new(&three);
        let topic = ra.switch_topic();
        let (tx, rx) = mpsc::channel(16);
        let rap = NtpServiceProxy::new(rx, ra)?;
        let executor = Executor {
            topic: topic.clone(),
            tx,
        };
        let runner = Runner { topic, rap };
        Ok(Self { runner, executor })
    }
}

pub trait NtpServiceTrait {
    fn ntc_service(&mut self, three: &ThreeTuple) -> Result<Runner>;
}

impl NtpServiceTrait for crate::MqttClient {
    fn ntc_service(&mut self, three: &ThreeTuple) -> Result<Runner> {
        let ra = NtpService::new(&three)?;
        self.executors
            .push(Box::new(ra.executor) as Box<dyn crate::Executor>);
        Ok(ra.runner)
    }
}
