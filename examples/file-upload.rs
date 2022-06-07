use aiot::{
    dm::{recv::RecvEnum, AsyncServiceInvoke},
    DataModelOptions, MqttClient, ThreeTuple,
};
use anyhow::Result;
use log::*;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple::from_env();
    let mut conn = MqttClient::new_public_tls(host, &three)?.connect();

    // let topic = format!("/sys/+/+/thing/file/upload/mqtt/init_reply");
    // conn.mqtt.subscribe(&topic, rumqttc::QoS::AtMostOnce).await?;
    let mut uploader = conn.file_uploader()?;
    uploader.init().await?;

    let options = DataModelOptions::new();
    let mut dm = conn.data_model(options)?;

    loop {
        tokio::select! {
            Ok(notification) = conn.poll() => {
                // 主循环的 poll 是必须的
                info!("Received = {:?}", notification);
            },
            Ok(data) = dm.poll() => {
                info!("物模型 = {:?}", data);
                match data {
                    RecvEnum::Service(AsyncServiceInvoke {msg_id:_, service_id, params}) => {
                        match service_id.as_str() {
                            "upload_file" => {
                                // let topic = format!("/sys/{}/{}/thing/file/upload/mqtt/init", three.product_key, three.device_name);
                                // let payload = r#"{"id":"1","params":{"fileName":"README.md","fileSize":-1,"conflictStrategy":"overwrite"}}"#;
                                // conn.mqtt.publish(&topic, rumqttc::QoS::AtMostOnce, false, payload).await?;
                                let path = params["path"].as_str().unwrap_or("./README.md");
                                uploader.upload(path).await?;
                            },
                            _ => {},
                        }
                    },
                    _ => {}
                }
            },
            Ok(recv) = uploader.poll() => {
                info!("{:?}", recv);
            }
        }
    }
}
