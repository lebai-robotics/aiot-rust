use anyhow::Result;
use log::*;

use aiot::ota::*;
use aiot::{MqttClient, ThreeTuple};
use recv_dto::{OTARecv};
#[tokio::main]
async fn main() -> Result<()> {
	env_logger::init();

	let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
	let three = ThreeTuple::from_env();
	let mut client = MqttClient::new_public_tls(&host, &three)?;

	let ota = client.ota()?;
	let (client, mut eventloop) = client.connect();
	let mut ota = ota.init(&client).await?;

	ota.report_version(String::from("0.0.0"), None).await?;

	ota.query_firmware(None).await?;
	loop {
		tokio::select! {
			 Ok(notification) = eventloop.poll() => {
				  // 主循环的 poll 是必须的
				  info!("Received = {:?}", notification);
			 }
			 Ok(recv) = ota.poll() => {
				 match recv {
					  OTARecv::UpgradePackageRequest(request) => {
						if let Some(package) = request.data{
							let data = ota.download_upgrade_package(&package).await?;

							ota.report_version(package.version.clone(), package.module.clone()).await?;
						}
						//  info!("data:{:?}",data);
					 },
					 OTARecv::GetFirmwareReply(request) => {
						if let Some(package) = request.data{
							let data = ota.download_upgrade_package(&package).await?;

							ota.report_version(package.version.clone(), package.module.clone()).await?;
							info!("data:{:?}",data);
						}
					},
				 }
			 }
		}
	}
}
