use anyhow::Result;
use log::*;
use serde_json::json;

use aiot::shadow;
use aiot::{MqttClient, ThreeTuple};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
   env_logger::init();

   let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
   let three = ThreeTuple::from_env();
   let mut client = MqttClient::new_public_tls(&host, &three)?;

   let shadow = client.shadow()?;
   let (client, mut eventloop) = client.connect();
   let mut shadow = shadow.init(&client).await?;

   // shadow
   //    .update(
   //       json!({
   //          "reported": {
   //             "p":10
   //          }
   //       }),
   //       3,
   //    )
   //    .await?;
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
          Ok(notification) = eventloop.poll() => {
              // 主循环的 poll 是必须的
              info!("Received = {:?}", notification);
          }
          Ok(recv) = shadow.poll() => {
             match recv {
               shadow::recv::ShadowRecv::ShadowGetTopic(response) => {
                  info!("ShadowGetTopic");
               },
            }
          }
      }
   }
}
