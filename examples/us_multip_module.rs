use aiot::bootstrap::recv::BootstrapRecv;
use aiot::mqtt::MqttConnection;
use aiot::subdev::base::{DeviceInfoId, DeviceInfoWithSecret};
use aiot::tag::base::{DeviceInfoKey, DeviceInfoKeyValue};
use anyhow::Result;
use log::*;
use tag::recv::{TagRecv};
use aiot::tag;
use aiot::{MqttClient, ThreeTuple};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple::from_env();
    let mut mqtt_connection = MqttConnection::new(MqttClient::new_public_tls(&host, &three)?);
    let mut tag = mqtt_connection.tag()?;
    let mut bootstrap = mqtt_connection.bootstrap()?;
    tag.update(vec![
        DeviceInfoKeyValue {
            attr_key: String::from("key1"),
            attr_value: String::from("key1_v"),
        },
        DeviceInfoKeyValue {
            attr_key: String::from("key2"),
            attr_value: String::from("key2_v"),
        },
    ], true).await?;


    loop {
        tokio::select! {
          Ok(notification) = mqtt_connection.poll() => {
              // 主循环的 poll 是必须的
              info!("Received = {:?}", notification);
          }
          Ok(recv) = tag.poll() => {
				 match recv {
               TagRecv::DeviceInfoUpdateResponse(_) => {

               },
               TagRecv::DeviceInfoDeleteResponse(_) => {
               },
            }
          }
          Ok(recv) = bootstrap.poll() => {
				 match recv {
               BootstrapRecv::BootstrapNotify(_) => {
                  
               },
            }
          }
      }
    }
}
