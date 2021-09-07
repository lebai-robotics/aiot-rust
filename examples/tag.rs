use aiot::subdev::base::{DeviceInfoId, DeviceInfoWithSecret};
use aiot::tag::base::{DeviceInfoKey, DeviceInfoKeyValue};
use anyhow::Result;
use log::*;
use tag::recv::{TagRecv};
use aiot::tag;
use aiot::{MqttClient, ThreeTuple};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
   env_logger::init();

   let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
   let three = ThreeTuple::from_env();
   let mut client = MqttClient::new_public_tls(&host, &three)?;

   let tag = client.tag()?;
   let (client, mut eventloop) = client.connect();
   let mut tag = tag.init(&client).await?;
   
   // tag.update(vec![
   //    DeviceInfoKeyValue{
   //       attr_key: String::from("key1"),
   //       attr_value: String::from("key1_v"),
   //    },
   //    DeviceInfoKeyValue{
   //       attr_key: String::from("key2"),
   //       attr_value: String::from("key2_v"),
   //    },
   // ], true).await?;
   let deleted_keys = vec![
      "key1",
      "key2"
   ];
   tag.delete(&deleted_keys, true).await?;


   loop {
      tokio::select! {
          Ok(notification) = eventloop.poll() => {
              // 主循环的 poll 是必须的
              info!("Received = {:?}", notification);
          }
          Ok(recv) = tag.poll() => {
             
				 match recv {
               TagRecv::DeviceInfoUpdateResponse(response) => {
                  info!("DeviceInfoUpdateResponse");

               },
               TagRecv::DeviceInfoDeleteResponse(response) => {
                  info!("DeviceInfoDeleteResponse");
               },
            }
          }
      }
   }
}
