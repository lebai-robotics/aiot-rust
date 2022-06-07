use aiot::{MqttClient, ThreeTuple};
use anyhow::Result;
use log::*;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple::from_env();
    let mut conn = MqttClient::new_public_tls(host, &three)?.connect();

    let mut ntp = conn.ntp_service()?;
    ntp.init().await?;
    ntp.send().await?;

    loop {
        tokio::select! {
            Ok(notification) = conn.poll() => {
                // 主循环的 poll 是必须的
                info!("Received = {:?}", notification);
            },
            Ok(recv) = ntp.poll() => {
                info!("{:?}", recv);
                let now = recv.calc().await?;
                info!("需要设置时间为 {:?}",now);
            }
        }
    }
}
