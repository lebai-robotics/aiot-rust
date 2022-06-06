use aiot::{MqttClient, ThreeTuple};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple::from_env();
    let mut conn = MqttClient::new_public_tls(host, &three)?.connect();
    let _bootstrap = conn.bootstrap()?;
    Ok(())
}
