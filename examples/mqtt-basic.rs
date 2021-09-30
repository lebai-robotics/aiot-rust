use aiot::{MqttClient, ThreeTuple};
use anyhow::Result;
use rumqttc::QoS;
use tokio::task;

#[tokio::main]
async fn main() -> Result<()> {
    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple::from_env();
    let (client, mut eventloop) = MqttClient::new_public_tls(host, &three)?.connect();
    client
        .subscribe(
            "/sys/a13FN5TplKq/mqtt_basic_demo/thing/event/+/post_reply",
            QoS::AtMostOnce,
        )
        .await?;

    task::spawn(async move {
        client
            .publish(
                "/sys/a13FN5TplKq/mqtt_basic_demo/thing/event/property/post",
                QoS::AtMostOnce,
                false,
                b"{\"id\":\"1\",\"version\":\"1.0\",\"params\":{\"LightSwitch\":0}}".to_vec(),
            )
            .await
            .unwrap();
    });

    loop {
        let notification = eventloop.poll().await?;
        println!("Received = {:?}", notification);
    }
}
