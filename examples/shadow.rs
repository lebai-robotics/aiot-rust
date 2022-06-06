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
    let mut conn = MqttClient::new_public_tls(host, &three)?.connect();
    let mut shadow = conn.shadow()?;

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
            Ok(notification) = conn.poll() => {
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
