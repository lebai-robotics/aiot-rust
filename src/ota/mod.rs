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
use crate::ota::recv::{PackageData, RecvEnum, UpgradePackageRequest, GetFirmwareReply};
use std::collections::HashMap;
use crate::http_downloader::{HttpDownloader, HttpDownloadConfig};
use crate::ota::msg::{OTAMsg, ReportProgress, ReportVersion, QueryFirmware};
use tempdir::TempDir;
use std::fs;
use crypto::digest::Digest;
use std::io::Read;
use crate::util::auth::sign;

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
		debug!("publish: {} {}",topic, String::from_utf8_lossy(&payload));
		self.client
			.publish(topic, QoS::AtMostOnce, false, payload)
			.await?;
		Ok(())
	}

	pub async fn report_version(&mut self, version: String, module: Option<String>) -> Result<()> {
		self.send(OTAMsg::report_version(ReportVersion {
			module,
			version,
		})).await?;
		Ok(())
	}
	pub async fn report_process(&mut self, report_process: ReportProgress) -> Result<()> {
		self.send(OTAMsg::report_process(report_process)).await?;
		Ok(())
	}

	pub async fn query_firmware(&mut self, module: Option<String>) -> Result<()> {
		self.send(OTAMsg::query_firmware(QueryFirmware {
			module,
		})).await?;
		Ok(())
	}

	pub async fn receive_upgrade_package(&mut self, request: &UpgradePackageRequest) -> Result<String> {
		debug!("start receive_upgrade_package");
		let module = request.data.module.clone();
		let version = request.data.version.clone();
		let tmp_dir = TempDir::new("ota")?;
		let file_path = tmp_dir.path()
			.join(format!("{}_{}", module.clone().unwrap_or(String::from("default")), version));
		let downloader = HttpDownloader::new(HttpDownloadConfig {
			block_size: 8000000,
			uri: request.data.url.clone(),
			file_path: String::from(file_path.to_str().unwrap()),
		});
		let results = futures_util::future::join(
			async {
				let process_receiver = downloader.get_process_receiver();
				let mut mutex_guard = process_receiver.lock().await;
				if let Some(download_process) = mutex_guard.recv().await {
					let report_progress = ReportProgress {
						module: module.clone(),
						desc: String::from(""),
						step: ((download_process.percent * 100f64) as u32).to_string(),
					};
					debug!("report_process finished {}",report_progress.step);
					self.report_process(report_progress);
				}
			},
			downloader.start(),
		).await;
		let mut ota_file_path = results.1?;
		let mut buffer = fs::read(ota_file_path.clone())?;
		debug!("size:{}",buffer.len());
		match request.data.sign_method.as_str() {
			"SHA256" => {
				let mut sha256 = crypto::sha2::Sha256::new();
				sha256.input(&buffer);
				let computed_result = sha256.result_str();
				if computed_result != request.data.sign {
					debug!("computed_result:{} sign:{}",computed_result,request.data.sign);
					return Err(Error::FileValidateFailed);
				}
			}
			"Md5" => {
				let mut md5 = crypto::md5::Md5::new();
				md5.input(&buffer);
				let computed_result = md5.result_str();
				if computed_result != request.data.sign {
					debug!("computed_result:{} sign:{}",computed_result,request.data.sign);
					return Err(Error::FileValidateFailed);
				}
			}
			_ => {
				return Err(Error::FileValidateFailed);
			}
		}

		std::fs::remove_file(file_path);
		std::fs::remove_dir_all(tmp_dir);
		debug!("receive_upgrade_package finished");
		Ok(ota_file_path)
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
		debug!("receive: {} {}", topic, String::from_utf8_lossy(payload));
		// "/ota/device/upgrade/+/+",
		if let Some(caps) = self.regs[0].captures(topic) {
			if &caps[1] != self.three.product_key || &caps[2] != self.three.device_name {
				return Ok(());
			}
			let payload: UpgradePackageRequest = serde_json::from_slice(&payload)?;
			info!("UpgradePackageRequest {:?}",&payload);
			let data = recv::OTARecv {
				device_name: caps[1].to_string(),
				product_key: caps[2].to_string(),
				data: RecvEnum::UpgradePackageRequest(payload),
			};
			self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
			return Ok(());
		}
		// "/sys/+/+/thing/ota/firmware/get_reply",
		if let Some(caps) = self.regs[1].captures(topic) {
			if &caps[1] != self.three.product_key || &caps[2] != self.three.device_name {
				return Ok(());
			}
			let payload: GetFirmwareReply = serde_json::from_str(&String::from_utf8_lossy(payload).replace(":{},", ":null,"))?;
			info!("GetFirmwareReply {:?}",&payload);
			let data = recv::RecvEnum::GetFirmwareReply(payload);
			let data = recv::OTARecv {
				device_name: caps[1].to_string(),
				product_key: caps[2].to_string(),
				data,
			};
			self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
			return Ok(());
		}

		Ok(())
	}
}

const TOPICS: &'static [&str] = &[
	"/ota/device/upgrade/+/+",
	"/sys/+/+/thing/ota/firmware/get_reply",
];

pub trait OTA {
	fn ota(&mut self, options: &OTAOptions) -> Result<HalfRunner>;
}

impl OTA for crate::MqttClient {
	fn ota(&mut self, options: &OTAOptions) -> Result<HalfRunner> {
		let regs = vec![
			Regex::new(r"/ota/device/upgrade/(.*)/(.*)")?,
			Regex::new(r"/sys/(.*)/(.*)/thing/ota/firmware/get_reply")?,
		];
		let (tx, rx) = mpsc::channel(64);
		let executor = Executor { tx, three: self.three.clone(), regs };

		self.executors
			.push(Box::new(executor) as Box<dyn crate::Executor>);
		let runner = HalfRunner {
			rx,
			three: self.three.clone(),
		};

		Ok(runner)
	}
}
