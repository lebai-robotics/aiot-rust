use std::sync::Arc;

use aiot::mqtt::MqttConnection;
use aiot::tag;
use aiot::{MqttClient, ThreeTuple};
use anyhow::Result;
use log::*;
use tag::recv::TagRecv;

#[tokio::main]
async fn main() -> Result<()> {
    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple::from_env();
    let mut mqtt_connection = MqttConnection::new(MqttClient::new_public_tls(&host, &three)?);
    let bootstrap = mqtt_connection.bootstrap()?;
    Ok(())
}
