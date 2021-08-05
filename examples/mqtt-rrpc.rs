use aiot::{MqttClient, ThreeTuple};
use anyhow::Result;
use log::*;
use rumqttc::{Event, Packet, QoS};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple::from_env();
    let (client, mut eventloop) = MqttClient::new_public_tls(&host, &three)?.connect();
    client
        .subscribe("/a13FN5TplKq/mqtt_rrpc_demo/user/get", QoS::AtMostOnce)
        .await?;

    loop {
        if let Ok(event) = eventloop.poll().await {
            match event {
                Event::Incoming(incoming) => {
                    info!("incoming = {:?}", incoming);
                    match incoming {
                        Packet::Publish(data) => {
                            // 下面是一个rrpc的应答示例
                            let payload = "pong";
                            client
                                .publish(&data.topic, QoS::AtMostOnce, false, payload)
                                .await?;
                        }
                        _ => {}
                    }
                }
                Event::Outgoing(outgoing) => {
                    info!("outgoing = {:?}", outgoing);
                }
            }
        }
    }
}
