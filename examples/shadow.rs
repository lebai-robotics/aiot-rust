use aiot::subdev::base::{DeviceInfoId, DeviceInfoWithSecret};
use anyhow::Result;
use log::*;
use regex::internal::Input;
use reqwest::Request;
use rumqttc::Event;
use serde_json::json;

use aiot::http_downloader::{HttpDownloadConfig, HttpDownloader};
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
   //          },
   //          "desired": {}
   //       }),
   //       1,
   //    )
   //    .await?;
   // shadow.get().await?;
   shadow
      .delete(
         json!({
            "reported":{
               "p":"null"
            }
         }),
         2,
      )
      .await?;

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
