use aiot::{MqttClient, ThreeTuple};
use anyhow::Result;
use log::*;
use regex::Regex;
use rumqttc::{Event, Packet, QoS};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple {
        product_key: "a13FN5TplKq".to_string(),
        device_name: "mqtt_broadcast_demo".to_string(),
        device_secret: "IQDVHqMEScjRj7DxYABKSkw8L5duQKoI".to_string(),
    };
    let (client, mut eventloop) = MqttClient::new_public_tls(host, &three)?.connect();

    // 指定Topic的所有设备接收广播消息，需订阅该Topic
    let topic = format!("/broadcast/{}/custom", three.product_key);
    client.subscribe(&topic, QoS::AtMostOnce).await?;

    let broadcast = Regex::new(r"/sys/(.*)/(.*)/broadcast/request/(.*)").unwrap();

    loop {
        if let Ok(event) = eventloop.poll().await {
            match event {
                Event::Incoming(incoming) => {
                    info!("incoming = {:?}", incoming);
                    match incoming {
                        Packet::Publish(data) => {
                            if broadcast.is_match(&data.topic) {
                                let caps = broadcast.captures(&data.topic).unwrap();
                                info!(
                                    "收到广播消息 messageId={} payload={}",
                                    &caps[3],
                                    String::from_utf8_lossy(&data.payload)
                                );
                            } else if &data.topic == &topic {
                                info!(
                                    "收到指定Topic广播消息 payload={}",
                                    String::from_utf8_lossy(&data.payload)
                                );
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}
