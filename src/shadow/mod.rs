use crate::{Error, Result, ThreeTuple};
use log::*;
use rumqttc::{AsyncClient, QoS};
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use serde::Serialize;
use crate::shadow::recv::*;
use crate::shadow::push::*;
use enum_iterator::IntoEnumIterator;

pub mod push;
pub mod recv;
pub mod base;

type Recv = ShadowRecv;
type RecvKind = ShadowRecvKind;

impl crate::MqttClient {
	fn shadow(&mut self) -> Result<HalfRunner> {
		let (tx, rx) = mpsc::channel(64);
		let executor = Executor { tx, three: self.three.clone() };

		self.executors
			.push(Box::new(executor) as Box<dyn crate::Executor>);
		let runner = HalfRunner {
			rx,
			three: self.three.clone(),
		};
		Ok(runner)
	}
}

pub struct Executor {
	three: Arc<ThreeTuple>,
	tx: Sender<Recv>,
}

pub struct HalfRunner {
	rx: Receiver<Recv>,
	three: Arc<ThreeTuple>,
}

impl HalfRunner {
	pub async fn init(self, client: &AsyncClient) -> Result<Runner> {
		let mut client = client.clone();
		let mut topics = rumqttc::Subscribe::empty_subscribe();
		for item in RecvKind::into_enum_iter() {
			let topic = item.get_topic();
			topics.add(topic.topic.to_string(), QoS::AtMostOnce);
		}
		client.subscribe_many(topics.filters).await?;

		Ok(Runner {
			rx: self.rx,
			client,
			three: self.three.clone(),
		})
	}
}

pub struct Runner {
	rx: Receiver<Recv>,
	client: AsyncClient,
	three: Arc<ThreeTuple>,
}

impl Runner {
	pub async fn poll(&mut self) -> Result<Recv> {
		self.rx.recv().await.ok_or(Error::RecvTopicError)
	}

	pub async fn publish<T>(&self, topic: String, payload: &T) -> Result<()>
		where T: ?Sized + Serialize, {
		let payload = serde_json::to_vec(payload)?;
		debug!("payload={}", String::from_utf8_lossy(&payload));
		self.client
			.publish(topic, QoS::AtMostOnce, false, payload)
			.await?;
		Ok(())
	}
}
