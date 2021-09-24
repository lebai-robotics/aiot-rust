use aiot::remote_config;
use aiot::{MqttClient, ThreeTuple};
use anyhow::Result;
use log::*;
use remote_config::recv::RemoteConfigRecv;
use serde_json::json;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<()> {
   env_logger::init();
   let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
   let three = ThreeTuple::from_env();
   let mut client = MqttClient::new_public_tls(&host, &three)?;

   let remote_config = client.remote_config()?;
   let (client, mut eventloop) = client.connect();
   let mut remote_config = remote_config.init(&client).await?;

   remote_config.get(true).await?;

   loop {
      tokio::select! {
          Ok(notification) = eventloop.poll() => {
              // 主循环的 poll 是必须的
              info!("Received = {:?}", notification);
          }
          Ok(recv) = remote_config.poll() => {
             match recv {
               RemoteConfigRecv::RemoteConfigGetReply(response) => {
                  if let Some(config_info) = response.data{
                     let data = remote_config.download_config(&config_info).await?;
                     debug!("config: {}", String::from_utf8_lossy(&data));
                  }

               },
               RemoteConfigRecv::RemoteConfigPush(response)=>{
                  if let Some(config_info) = response.data{
                     let data = remote_config.download_config(&config_info).await?;
                     debug!("config: {}", String::from_utf8_lossy(&data));
                  }
               },
            }
          }
      }
   }
}
