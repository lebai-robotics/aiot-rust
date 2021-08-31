//! OTA

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
// use crate::alink_topic::AlinkTopic;
use crate::ota::recv::{UpgradePackage, RecvEnum};
use std::collections::HashMap;

/// OTA设置
#[derive(Debug, Clone)]
pub struct OTAOptions {}

impl OTAOptions {
	pub fn new() -> Self {
		Self {}
	}
}

pub struct HalfRunner {
	rx: Receiver<recv::OTARecv>,
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
			client,
			fs: vec![],
			three: self.three.clone(),
		})
	}
}

pub struct Runner {
	rx: Receiver<recv::OTARecv>,
	client: AsyncClient,
	three: Arc<ThreeTuple>,
}

pub struct ReceivedData {
	data: Vec<u8>,
	percent: f32,
}

impl Runner {
	pub async fn send(&mut self, data: msg::OTAMsg) -> Result<()> {
		let mut data = data;
		if data.product_key.is_none() {
			data.product_key = Some(self.three.product_key.to_string());
		}
		if data.device_name.is_none() {
			data.device_name = Some(self.three.device_name.to_string());
		}
		let (topic, payload) = data.to_payload(0)?;
		debug!("payload={}", String::from_utf8_lossy(&payload));
		self.client
			.publish(topic, QoS::AtMostOnce, false, payload)
			.await?;
		Ok(())
	}

	pub async fn poll(&mut self) -> Result<recv::OTARecv> {
		self.rx.recv().await.ok_or(Error::RecvTopicError)
	}
}

pub struct Executor {
	three: Arc<ThreeTuple>,
	tx: Sender<recv::OTARecv>,
	regs: Vec<Regex>,
}

#[async_trait::async_trait]
impl crate::Executor for Executor {
	async fn execute(&self, topic: &str, payload: &[u8]) -> Result<()> {
		debug!("{} {}", topic, String::from_utf8_lossy(payload));
		// "/ota/device/upgrade/+/+",
		if let Some(caps) = self.regs[0].captures(topic) {
			if &caps[0] != self.three.product_key || &caps[1] != self.three.device_name {
				return Ok(());
			}
			let payload: AlinkRequest<UpgradePackage> = serde_json::from_slice(&payload)?;
			let data = recv::OTARecv {
				device_name: caps[0].to_string(),
				product_key: caps[1].to_string(),
				data: RecvEnum::UpgradePackage(payload.params),
			};
			self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
			return Ok(());
		}
		// "/sys/+/+/thing/ota/firmware/get_reply",
		if let Some(caps) = self.regs[1].captures(topic) {
			if &caps[0] != self.three.product_key || &caps[1] != self.three.device_name {
				return Ok(());
			}
			let payload: AlinkRequest<UpgradePackage> = serde_json::from_slice(&payload)?;
			let data = recv::RecvEnum::UpgradePackage(payload.params);
			let data = recv::OTARecv {
				device_name: caps[0].to_string(),
				product_key: caps[1].to_string(),
				data,
			};
			self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
			return Ok(());
		}

		Ok(())
	}
}

struct OTAInner {
	runner: HalfRunner,
	executor: Executor,
}

const TOPICS: &'static [&str] = &[
	"/ota/device/upgrade/+/+",
	//
	// "/ota/device/inform/+/+",
	// "/ota/device/progress/+/+",
	// "/sys/+/+/thing/ota/firmware/get",
	// "/sys/+/+/thing/ota/firmware/get_reply",
];

impl OTAInner {
	fn new(options: &OTAOptions, three: Arc<ThreeTuple>) -> Result<Self> {
		let regs = vec![
			Regex::new(r"/ota/device/upgrade/+/+")?,
			Regex::new(r"/sys/+/+/thing/ota/firmware/get_reply")?,
			// Regex::new(r"/ota/device/inform/+/+")?,
			// Regex::new(r"/ota/device/progress/+/+")?,
			// Regex::new(r"/sys/+/+/thing/ota/firmware/get")?,
		];
		let (tx, rx) = mpsc::channel(64);
		let runner = HalfRunner {
			rx,
			three: three.clone(),
		};
		let executor = Executor { tx, three, regs };
		Ok(Self { runner, executor })
	}
}

pub trait OTA {
	fn ota(&mut self, options: &OTAOptions) -> Result<HalfRunner>;
}

impl OTA for crate::MqttClient {
	fn ota(&mut self, options: &OTAOptions) -> Result<HalfRunner> {
		let ra = OTAInner::new(&options, self.three.clone())?;
		self.executors
			.push(Box::new(ra.executor) as Box<dyn crate::Executor>);
		Ok(ra.runner)
	}
}
