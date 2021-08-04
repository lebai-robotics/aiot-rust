use aiot::{MqttClient, NtpServiceTrait, ThreeTuple};
use anyhow::Result;
use log::*;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple::from_env();
    let mut client = MqttClient::new_public_tls(&host, &three)?;

    let ntp = client.ntp_service()?;
    let (client, mut eventloop) = client.connect();
    let mut ntp = ntp.init(&client).await?;

    ntp.send().await?;

    loop {
        tokio::select! {
            Ok(notification) = eventloop.poll() => {
                // 主循环的 poll 是必须的
                info!("Received = {:?}", notification);
            },
            Ok(recv) = ntp.poll() => {
                info!("{:?}", recv);
                recv.sync().await?;
            }
        }
    }
}
