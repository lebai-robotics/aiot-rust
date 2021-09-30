use aiot::{Error, MqttClient, RemoteAccess, ThreeTuple};
use anyhow::Result;
use log::*;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple::from_env();
    let mut client = MqttClient::new_public_tls(&host, &three)?;

    let mut ra = client.remote_access()?;
    let (client, mut eventloop) = client.connect();
    ra.init(&client).await?;
    tokio::spawn(async move {
        loop {
            ra.poll().await?;
        }
        #[allow(unreachable_code)]
            Ok::<_, Error>(())
    });

    loop {
        tokio::select! {
            Ok(notification) = eventloop.poll() => {
                // 主循环的 poll 是必须的
                info!("Received = {:?}", notification);
            }
        }
    }
}
