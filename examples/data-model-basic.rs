use aiot::{DataModel, DataModelMsg, DataModelOptions, MqttClient, RecvEnum, ThreeTuple};
use anyhow::Result;
use log::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple::from_env();
    let mut client = MqttClient::new_public_tls(host, &three)?;

    let options = DataModelOptions::new();
    let dm = client.data_model(&options)?;
    let (client, mut eventloop) = client.connect();
    let mut dm = dm.init(&client).await?;

    dm.send(DataModelMsg::property_post(json!({
        "LightSwitch": 0
    })))
    .await?;
    dm.send(DataModelMsg::event_post(
        "Error".to_string(),
        json!({
            "ErrorCode": 0
        }),
    ))
    .await?;

    let mut history = Vec::new();
    history.push(json!({
      "identity": {
        "productKey": "",
        "deviceName": ""
      },
      "properties": [
        {
          "Power": {
            "value": "on",
            "time": 1524448722000u64
          },
          "WF": {
            "value": "3",
            "time": 1524448722000u64
          }
        },
        {
          "Power": {
            "value": "on",
            "time": 1524448722000u64
          },
          "WF": {
            "value": "3",
            "time": 1524448722000u64
          }
        }
      ],
      "events": [
        {
          "alarmEvent": {
            "value": {
              "Power": "on",
              "WF": "2"
            },
            "time": 1524448722000u64
          },
          "alertEvent": {
            "value": {
              "Power": "off",
              "WF": "3"
            },
            "time": 1524448722000u64
          }
        }
      ]
    }));
    history.push(json!({
      "identity": {
        "productKey": "",
        "deviceName": ""
      },
      "properties": [
        {
          "Power": {
            "value": "on",
            "time": 1524448722000u64
          },
          "WF": {
            "value": "3",
            "time": 1524448722000u64
          }
        }
      ],
      "events": [
        {
          "alarmEvent": {
            "value": {
              "Power": "on",
              "WF": "2"
            },
            "time": 1524448722000u64
          },
          "alertEvent": {
            "value": {
              "Power": "off",
              "WF": "3"
            },
            "time": 1524448722000u64
          }
        }
      ]
    }));
    dm.send(DataModelMsg::history_post(history)).await?;

    loop {
        tokio::select! {
            Ok(notification) = eventloop.poll() => {
                // 主循环的 poll 是必须的
                info!("Received = {:?}", notification);
            },
            Ok(recv) = dm.poll() => {
                match recv.data {
                    RecvEnum::PropertySet(data) => {
                        info!("属性设置 {:?}", data);
                        // 以下代码演示如何对来自云平台的属性设置指令进行应答
                        dm.send(DataModelMsg::property_set_reply(200, json!({}), data.msg_id)).await?;
                    },
                    RecvEnum::GenericReply(data) => {
                        // 属性上报, 事件上报, 获取期望属性值或者删除期望属性值的应答
                        info!("服务端应答 {:?}", data);
                    },
                    RecvEnum::AsyncServiceInvoke(data) => {
                        info!("异步服务调用 {:?}", data);
                        // 以下代码演示如何对来自云平台的异步服务调用进行应答
                        dm.send(DataModelMsg::async_service_reply(200, json!({"dataA": 20}), data.msg_id, data.service_id)).await?;
                    },
                    RecvEnum::SyncServiceInvoke(data) => {
                        info!("同步服务调用 {:?}", data);
                        // 以下代码演示如何对来自云平台的同步服务调用进行应答
                        dm.send(DataModelMsg::sync_service_reply(200, json!({}), data.rrpc_id, data.msg_id, data.service_id)).await?;
                    },
                    RecvEnum::RawData(data) => {
                        info!("下行二进制数据 {:?}", data);
                        // 以下代码演示如何发送二进制格式数据, 若使用需要有相应的数据透传脚本部署在云端
                        let raw = vec![0x01, 0x02];
                        dm.send(DataModelMsg::raw_data(raw)).await?;
                    },
                    RecvEnum::RawSyncServiceInvoke(data) => {
                        info!("二进制格式的同步服务调用 {:?}", data);
                        let raw = vec![0x01, 0x02];
                        dm.send(DataModelMsg::raw_service_reply(raw, data.rrpc_id)).await?;
                    },
                    RecvEnum::RawDataReply(data) => {
                        info!("上行二进制数据后, 云端的回复报文 {:?}", data);
                    },
                }
            }
        }
    }
}
