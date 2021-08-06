use aiot::{LogPost, MqttClient, ThreeTuple};
use anyhow::Result;
use log::*;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple::from_env();

    let mut client = MqttClient::new_public_tls(&host, &three)?;

    let log_post = client.log_post()?;

    let (client, mut eventloop) = client.connect();

    let mut log_post = log_post.init(&client).await?;

    loop {
        tokio::select! {
            Ok(event) = eventloop.poll() => {
                info!("{:?}", event);
            }
            Ok(log_config) = log_post.poll() => {
                info!("云端推送日志配置 {:?}", log_config);
            }
        }
    }
}
