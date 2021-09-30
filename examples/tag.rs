use aiot::mqtt::MqttConnection;

use aiot::tag;
use aiot::tag::base::DeviceInfoKeyValue;
use aiot::{MqttClient, ThreeTuple};
use anyhow::Result;
use log::*;
use tag::recv::TagRecv;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple::from_env();
    let mut mqtt_connection = MqttConnection::new(MqttClient::new_public_tls(host, &three)?);
    let mut tag = mqtt_connection.tag()?;
    tag.update(
        vec![
            DeviceInfoKeyValue {
                attr_key: String::from("key1"),
                attr_value: String::from("key1_v"),
            },
            DeviceInfoKeyValue {
                attr_key: String::from("key2"),
                attr_value: String::from("key2_v"),
            },
        ],
        true,
    )
    .await?;
    // let deleted_keys = vec![
    //    "key1",
    //    "key2"
    // ];
    // tag.delete(&deleted_keys, true).await?;

    loop {
        tokio::select! {
            Ok(notification) = mqtt_connection.poll() => {
                // 主循环的 poll 是必须的
                info!("Received = {:?}", notification);
            }
            Ok(recv) = tag.poll() => {
                   match recv {
                 TagRecv::DeviceInfoUpdateResponse(_response) => {
                    info!("DeviceInfoUpdateResponse");

                 },
                 TagRecv::DeviceInfoDeleteResponse(_response) => {
                    info!("DeviceInfoDeleteResponse");
                 },
              }
            }
        }
    }
}
