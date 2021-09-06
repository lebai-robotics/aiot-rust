use anyhow::Result;
use log::*;
use regex::internal::Input;
use reqwest::Request;
use rumqttc::Event;
use serde_json::json;

use aiot::http_downloader::{HttpDownloadConfig, HttpDownloader};
use aiot::subdev;
use aiot::subdev::base::{DeviceInfo, DeviceInfoId};
use aiot::subdev::push::LoginParam;
use aiot::subdev::recv_dto::*;
use aiot::{DataModel, DataModelMsg, DataModelOptions, MqttClient, ThreeTuple};
use chrono::Duration;
use futures_util::StreamExt;
use log::Level::Error;
use std::str::Bytes;
use std::sync::Arc;
use tempdir::TempDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	env_logger::init();

	let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
	let three = ThreeTuple::from_env();
	let mut client = MqttClient::new_public_tls(&host, &three)?;

	let subdev = client.subdev()?;
	let (client, mut eventloop) = client.connect();
	let mut subdev = subdev.init(&client).await?;

	let sub_device_id = DeviceInfoId {
		product_key: "gbgmHl8l0ee".to_string(),
		device_name: "subDevice".to_string(), // "ee1fe40b755a7034dadd0e47d69c83b7"
	};
	let sub_device_login = LoginParam {
		product_key: sub_device_id.product_key.clone(),
		device_name: sub_device_id.device_name.clone(),
		clean_session: false,
		device_secret: "9b559fb55e8c928537876d0f7aae6aaf".to_string(),
		// "ee1fe40b755a7034dadd0e47d69c83b7"
	};

	// 子设备上线
	subdev.login(sub_device_login.clone()).await?;
	// std::thread::sleep(core::time::Duration::from_secs(2));
	//
	// std::thread::sleep(core::time::Duration::from_secs(2));
	// // 子设备批量上线
	// let sub_devices = vec![
	// 	sub_device_login.clone()
	// ];
	// subdev.batch_login(&sub_devices).await?;
	// std::thread::sleep(core::time::Duration::from_secs(2));
	// // 子设备批量下线
	// let sub_devices = vec![sub_device_id.clone()];
	// subdev.batch_logoSubDevRecv::SubDevLogoutResponse { field1: data }ad::sleep(core::time::Duration::from_secs(2));
	// // 子设备禁用
	// subdev.disable(sub_device_id.clone()).await?;
	// std::thread::sleep(core::time::Duration::from_secs(2));
	// // 子设备启用
	// subdev.enable(sub_device_id.clone()).await?;
	// std::thread::sleep(core::time::Duration::from_secs(2));

	// 子设备禁用
	// subdev.disable(sub_device_id.clone()).await?;
	// 子设备下线
	// subdev.disable(sub_device_id.clone()).await?;

	loop {
		tokio::select! {
			 Ok(notification) = eventloop.poll() => {
				  // 主循环的 poll 是必须的
				  info!("Received = {:?}", notification);
			 }
			 Ok(recv) = subdev.poll() => {
				 match recv {
					 SubDevRecv::SubDevLoginResponse(_) => todo!(),
					 SubDevRecv::SubDevBatchLoginResponse(_) => todo!(),
					 SubDevRecv::SubDevLogoutResponse(_) => todo!(),
					 SubDevRecv::SubDevBatchLogoutResponse(_) => todo!(),
					 SubDevRecv::SubDevDisableResponse(_) => todo!(),
					 SubDevRecv::SubDevEnableResponse(_) => todo!(),
					 SubDevRecv::SubDevDeleteResponse(_) => todo!(),
					 SubDevRecv::SubDevAddTopologicalRelationResponse(_) => todo!(),
					 SubDevRecv::SubDevDeleteTopologicalRelationResponse(_) => todo!(),
					 SubDevRecv::SubDevGetTopologicalRelationResponse(_) => todo!(),
					 SubDevRecv::SubDevDeviceReportResponse(_) => todo!(),
					 SubDevRecv::SubDevAddTopologicalRelationNotifyRequest(_) => todo!(),
					 SubDevRecv::SubDevChangeTopologicalRelationNotifyRequest(_) => todo!(),
				 }
			 }
		}
	}
}
