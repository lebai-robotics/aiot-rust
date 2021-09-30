use aiot::mqtt::MqttConnection;
use aiot::subdev::base::DeviceInfoWithSecret;
use aiot::subdev::recv::SubDevRecv;
use anyhow::Result;
use log::*;

use aiot::subdev::base::{DeviceInfo, DeviceInfoId};
use aiot::subdev::push::LoginParam;
use aiot::{MqttClient, ThreeTuple};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple::from_env();

    let mut mqtt_connection = MqttConnection::new(MqttClient::new_public_tls(&host, &three)?);
    let mut subdev = mqtt_connection.subdev()?;

    let sub_device_id = DeviceInfoId {
        product_key: "gbgmHl8l0ee".to_string(),
        device_name: "subDevice".to_string(),
    };
    let sub_device_id2 = DeviceInfoId {
        product_key: "gbgmHl8l0ee".to_string(),
        device_name: "abcd".to_string(),
    };
    let sub_device_with_secret = DeviceInfoWithSecret {
        product_key: sub_device_id.product_key.clone(),
        device_name: sub_device_id.device_name.clone(),
        device_secret: "9b559fb55e8c928537876d0f7aae6aaf".to_string(), // "eeee1fe40b755a7034dadd0e47d69c83b7"
    };
    let sub_device_with_secret2 = DeviceInfoWithSecret {
        product_key: sub_device_id2.product_key.clone(),
        device_name: sub_device_id2.device_name.clone(),
        device_secret: "9c0adcc900f00a14f32ceb18c1efe589".to_string(), // "eeee1fe40b755a7034dadd0e47d69c83b7"
    };
    let sub_device_login = LoginParam {
        product_key: sub_device_id.product_key.clone(),
        device_name: sub_device_id.device_name.clone(),
        clean_session: false,
        device_secret: sub_device_with_secret.device_secret.clone(),
    };
    let sub_devices = vec![sub_device_login.clone()];
    let sub_device_ids = vec![sub_device_id.clone()];
    let sub_device_witch_secrets = vec![sub_device_with_secret2.clone()];

    // subdev
    // 	.get_topological_relation(
    // 		true,
    // 	)
    // 	.await?;

    // subdev.delete_topological_relation(&sub_device_ids.clone(), true).await?;
    // subdev.add_topological_relation(&sub_device_witch_secrets.clone(), true).await?;
    let sub_device_ids3 = vec![DeviceInfoId {
        product_key: "gbgmHl8l0ee".to_string(),
        device_name: "aaaa".to_string(),
    }];
    subdev.register(&sub_device_ids3, true).await?;
    // subdev.found_report(&sub_device_ids.clone(), true).await?;

    // 子设备上线
    // subdev.login(sub_device_login.clone()).await?;
    // subdev.batch_login(&sub_devices.clone()).await?;

    loop {
        tokio::select! {
			 Ok(notification) = mqtt_connection.poll() => {
				  // 主循环的 poll 是必须的
				  info!("Received = {:?}", notification);
			 }
			 Ok(recv) = subdev.poll() => {
				 match recv {
					SubDevRecv::SubDevLoginResponse(_) => {},
					SubDevRecv::SubDevBatchLoginResponse(_) => {},
					SubDevRecv::SubDevLogoutResponse(_) => {},
					SubDevRecv::SubDevBatchLogoutResponse(_) => {},
					SubDevRecv::SubDevDisableResponse(response) => {
						subdev.disable_reply(response.id,200).await?;
						info!("SubDevDisableResponse");
					},		SubDevRecv::SubDevEnableResponse(response) => {
						subdev.enable_reply(response.id,200).await?;
						info!("SubDevEnableResponse");
					},
					SubDevRecv::SubDevDeleteResponse(response) => {
						subdev.delete_reply(response.id,200).await?;
						info!("SubDevDeleteResponse");
					},
					SubDevRecv::SubDevAddTopologicalRelationResponse(_) => {},
					SubDevRecv::SubDevDeleteTopologicalRelationResponse(_) => {},
					SubDevRecv::SubDevGetTopologicalRelationResponse(_) => {},
					SubDevRecv::SubDevDeviceReportResponse(_) => {},
					SubDevRecv::SubDevAddTopologicalRelationNotifyRequest(_) => {},
					SubDevRecv::SubDevChangeTopologicalRelationNotifyRequest(_) => {},
					SubDevRecv::SubDevRegisterResponse(response) => {
						if let Some(data) = response.data{
							let r:Vec<DeviceInfoWithSecret> =data
							.iter()
							.map(|n|n.clone().into())
							.collect();
							subdev.add_topological_relation(&r, true).await?;
						}
					},
				}
			}
		}
    }
}
