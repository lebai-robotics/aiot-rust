use aiot::{ota::recv::OTARecv, MqttClient, ThreeTuple};
use anyhow::Result;
use log::*;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple::from_env();

    let mut conn = MqttClient::new_public_tls(host, &three)?.connect();
    let mut ota = conn.ota()?;

    ota.report_version("0.0.0".to_string(), None).await?;

    ota.query_firmware(None).await?;
    loop {
        tokio::select! {
             Ok(notification) = conn.poll() => {
                  // 主循环的 poll 是必须的
                  info!("Received = {:?}", notification);
             }
             Ok(recv) = ota.poll() => {
                 match recv {
                      OTARecv::UpgradePackageRequest(request) => {
                        if let Some(package) = request.data{
                            info!("{package:?}");
                            let _data = ota.download_to(&package, "tmp1.tar.gz").await?;
                            // ota.report_version(&package.version, package.module.as_ref().map(|x| &**x)).await?;
                        }
                     },
                     OTARecv::GetFirmwareReply(request) => {
                        if let Some(package) = request.data{
                            info!("{package:?}");
                            let _data = ota.download_to(&package, "temp2.tar.gz").await?;
                            // ota.report_version(&package.version, package.module.as_ref().map(|x| &**x)).await?;
                        }
                    },
                 }
             }
        }
    }
}
