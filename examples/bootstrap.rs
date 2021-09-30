use aiot::mqtt::MqttConnection;
use aiot::tag;
use aiot::{MqttClient, ThreeTuple};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple::from_env();
    let mut mqtt_connection = MqttConnection::new(MqttClient::new_public_tls(host, &three)?);
    let _bootstrap = mqtt_connection.bootstrap()?;
    Ok(())
}
