use aiot::{DataModelMsg, DataModelOptions, DataModelTrait, MqttClient, MsgEnum, ThreeTuple};
use anyhow::Result;
use log::*;
use serde_json::{Map, Value, json};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple::from_env();
    let mut client = MqttClient::new_public_tls(&host, &three)?;

    let options = DataModelOptions::new();
    let dm = client.data_model(&options)?;
    let (client, mut eventloop) = client.connect();
    let mut dm = dm.init(client.clone()).await?;

    let mut params: Map<String, Value> = Map::new();
    params.insert("LightSwitch".to_string(), json!(0));
    let data = DataModelMsg::new(MsgEnum::new_property_post(params));
    dm.send(data).await?;

    loop {
        tokio::select! {
            Ok(notification) = eventloop.poll() => {
                // 主循环的 poll 是必须的
                info!("Received = {:?}", notification);
            }
        }
    }
}
