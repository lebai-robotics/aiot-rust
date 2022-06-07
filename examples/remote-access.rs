use aiot::{Error, MqttClient, ThreeTuple};
use anyhow::Result;
use log::*;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple::from_env();
    let mut conn = MqttClient::new_public_tls(host, &three)?.connect();

    let (ra, mut rap) = conn.remote_access()?;
    tokio::spawn(async move {
        ra.init().await?;
        loop {
            rap.poll().await?;
        }
        #[allow(unreachable_code)]
        Ok::<_, Error>(())
    });

    loop {
        tokio::select! {
            Ok(notification) = conn.poll() => {
                // 主循环的 poll 是必须的
                info!("Received = {:?}", notification);
            }
        }
    }
}
