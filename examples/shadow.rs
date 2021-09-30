use aiot::mqtt::MqttConnection;
use anyhow::Result;
use log::*;

use aiot::shadow;
use aiot::{MqttClient, ThreeTuple};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple::from_env();
    let mut mqtt_connection = MqttConnection::new(MqttClient::new_public_tls(host, &three)?);
    let mut shadow = mqtt_connection.shadow()?;

    shadow
        .update(
            json!({
               "reported": {
                  "p":10
               }
            }),
            3,
        )
        .await?;
    shadow.get().await?;
    // shadow
    //    .delete(
    //       json!({
    //          "reported":{
    //             "p":"null"
    //          }
    //       }),
    //       4,
    //    )
    //    .await?;

    loop {
        tokio::select! {
            Ok(notification) = mqtt_connection.poll() => {
                // 主循环的 poll 是必须的
                info!("Received = {:?}", notification);
            }
            Ok(recv) = shadow.poll() => {
               match recv {
                 shadow::recv::ShadowRecv::ShadowGetTopic(_response) => {
                    info!("ShadowGetTopic");
                 },
              }
            }
        }
    }
}
