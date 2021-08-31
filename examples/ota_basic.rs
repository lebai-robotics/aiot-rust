use anyhow::Result;
use log::*;
use regex::internal::Input;
use reqwest::Request;
use rumqttc::Event;
use serde_json::json;

use aiot::{DataModel, DataModelMsg, DataModelOptions, MqttClient, ThreeTuple};
use aiot::ota::{OTA, OTAOptions};
use aiot::ota::msg::{MsgEnum, OTAMsg, ReportProgress, ReportVersion};
use aiot::ota::recv::RecvEnum;
use futures_util::StreamExt;
use std::str::Bytes;
use aiot::http_downloader::{HttpDownloader, HttpDownloadConfig};
use std::sync::Arc;
use tempdir::TempDir;
use log::Level::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	env_logger::init();

	/*	let downloader = HttpDownloader::new(HttpDownloadConfig {
			block_size: 8000000,
			uri: "https://9c9475b931e347a2760e3f997d8a68a2.dlied1.cdntips.net/dlied1.qq.com/qqweb/PCQQ/PCQQ_EXE/PCQQ2021.exe?mkey=612d874974e972ba&f=07b4&cip=116.233.84.79&proto=https&access_type=$header_ApolloNet".to_string(),
			path: "C:/Users/19743/Desktop/Test/".to_string(),
			file_name: "qq.exe".to_string(),
		});
		let data = downloader.start().await?;*/

	let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
	let three = ThreeTuple::from_env();
	let mut client = MqttClient::new_public_tls(&host, &three)?;

	let options = OTAOptions::new();
	let dm = client.ota(&options)?;
	let (client, mut eventloop) = client.connect();
	let mut ota = dm.init(&client).await?;

	ota.send(OTAMsg::report_version(ReportVersion {
		module: None,
		version: String::from(""),
	}));

	loop {
		let notification = eventloop.poll().await?;
		info!("Received = {:?}", notification);
		if let Ok(rev) = ota.poll().await {
			match rev.data {
				RecvEnum::UpgradePackage(data) => {
					let tmp_dir = TempDir::new("ota")?;
					let file_path = tmp_dir.path().join(format!("{}_{}", data.module, data.version));
					let downloader = HttpDownloader::new(HttpDownloadConfig {
						block_size: 8000000,
						uri: data.url,
						file_path: String::from(file_path.to_str().unwrap()),
					});
					let results = futures_util::future::join(
						async {
							let process_receiver = downloader.get_process_receiver();
							let mut mutex_guard = process_receiver.lock().await;
							if let Some(download_process) = mutex_guard.recv().await {
								ota.send(OTAMsg::report_process(ReportProgress {
									module: "".to_string(),
									desc: String::from(""),
									step: ((download_process.percent * 100f64) as u32).to_string(),
								}));
							}
						},
						downloader.start(),
					).await;
					let data = results.1?;
				}
				RecvEnum::GetFirmwareReply(data) => {}
			}
		}
	}
}
