use aiot::{MqttClient, ThreeTuple};
use anyhow::Result;
use log::*;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple::from_env();

    let mut conn = MqttClient::new_public_tls(host, &three)?.connect();
    let mut log_post = conn.log_post()?;

    log_post.get("device", "content").await?;
    loop {
        tokio::select! {
            Ok(event) = conn.poll() => {
                info!("{:?}", event);
            }
            Ok(log_config) = log_post.poll() => {
                info!("云端推送日志配置 {:?}", log_config);
            }
        }
    }
}
