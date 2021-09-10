use serde_json::Value;
use aiot::remote_config;
use aiot::{MqttClient, ThreeTuple};
use anyhow::Result;
use log::*;
use remote_config::recv::RemoteConfigRecv;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
						 let data = remote_config.download_config(&response.data).await?;
                   debug!("config: {}", String::from_utf8_lossy(&data));

               },
               RemoteConfigRecv::RemoteConfigPush(response)=>{
                  let data = remote_config.download_config(&response.data).await?;
                  debug!("config: {}", String::from_utf8_lossy(&data));
               },
            }
          }
      }
   }
}
