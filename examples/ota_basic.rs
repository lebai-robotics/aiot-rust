use anyhow::Result;
use log::*;
use reqwest::Request;
use rumqttc::Event;
use serde_json::json;

use aiot::http_downloader::{HttpDownloadConfig, HttpDownloader};
use aiot::ota::*;
use aiot::{DataModel, DataModelMsg, DataModelOptions, MqttClient, ThreeTuple};
use futures_util::StreamExt;
use log::Level::Error;
use std::str::Bytes;
use std::sync::Arc;
use tempdir::TempDir;

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

	let ota = client.ota()?;
	let (client, mut eventloop) = client.connect();
	let mut ota = ota.init(&client).await?;

	ota.report_version(String::from("0.0.0"), None).await?;

	// ota.query_firmware(None).await?;

	loop {
		tokio::select! {
			 Ok(notification) = eventloop.poll() => {
				  // 主循环的 poll 是必须的
				  info!("Received = {:?}", notification);
			 }
			 Ok(recv) = ota.poll() => {
				 match recv {
					  recv_dto::OTARecv::UpgradePackageRequest(request) => {

						 let file = ota.receive_upgrade_package(&request).await?;

						 ota.report_version(request.data.version.clone(), request.data.module.clone()).await?;
						 info!("file:{:?}",file);
					 },
					 recv_dto::OTARecv::GetFirmwareReply(request) => {},
				 }
			 }
		}
	}
}
