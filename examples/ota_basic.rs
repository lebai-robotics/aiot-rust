use anyhow::Result;
use log::*;

use aiot::{mqtt::MqttConnection, ota::recv::OTARecv, MqttClient, ThreeTuple};
#[tokio::main]
async fn main() -> Result<()> {
	env_logger::init();

	let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
	let three = ThreeTuple::from_env();

	let mut mqtt_connection = MqttConnection::new(MqttClient::new_public_tls(&host, &three)?);
	let mut ota = mqtt_connection.ota()?;

	ota.report_version(String::from("0.0.0"), None).await?;

	ota.query_firmware(None).await?;
	loop {
		tokio::select! {
			 Ok(notification) = mqtt_connection.poll() => {
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
