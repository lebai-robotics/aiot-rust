use crate::alink::*;
use crate::{Error, Result, ThreeTuple};
use log::*;
use regex::Regex;
use rumqttc::{AsyncClient, QoS};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
// use crate::alink_topic::AlinkTopic;
use std::collections::HashMap;
use crate::subdev::msg::{SubDevLoginRequest, DeviceInfo, SubDevBatchLoginRequest, SubDevBatchLoginParams, SubDevLogoutRequest, SubDevBatchLogoutRequest, SubDevMethodRequest, SubDevAddTopologicalRelationRequest, SubDevDeleteTopologicalRelationRequest, SubDevGetTopologicalRelationRequest, SubDevFoundReportRequest};
use crate::subdev::recv::DeviceInfoId;
use serde::Serialize;

pub mod msg;
pub mod recv;

impl crate::MqttClient {
	fn subdev(&mut self) -> Result<HalfRunner> {
		let (tx, rx) = mpsc::channel(64);
		let executor = Executor { tx, three: self.three.clone() };

		self.executors
			.push(Box::new(executor) as Box<dyn crate::Executor>);
		let runner = HalfRunner {
			rx,
			three: self.three.clone(),
		};
		Ok(runner)
	}
}

pub struct HalfRunner {
	rx: Receiver<recv::SubDevRecv>,
	three: Arc<ThreeTuple>,
}

impl HalfRunner {
	pub async fn init(self, client: &AsyncClient) -> Result<AlinkOTA> {
		let mut client = client.clone();
		let mut topics = rumqttc::Subscribe::empty_subscribe();
		// for &topic in TOPICS {
		// 	topics.add(topic.to_string(), QoS::AtMostOnce);
		// }
		client.subscribe_many(topics.filters).await?;
		Ok(AlinkOTA {
			rx: self.rx,
			client,
			three: self.three.clone(),
		})
	}
}

pub struct AlinkOTA {
	rx: Receiver<recv::SubDevRecv>,
	client: AsyncClient,
	three: Arc<ThreeTuple>,
}

pub struct LoginParam {
	pub product_key: String,
	pub device_name: String,
	pub clean_session: bool,
}

impl AlinkOTA {
	pub async fn poll(&mut self) -> Result<recv::SubDevRecv> {
		self.rx.recv().await.ok_or(Error::RecvTopicError)
	}

	pub async fn publish<T>(&self, topic: String, payload: &T) -> Result<()>
		where T: ?Sized + Serialize, {
		let payload = serde_json::to_vec(payload)?;
		debug!("payload={}", String::from_utf8_lossy(&payload));
		self.client
			.publish(topic, QoS::AtMostOnce, false, payload)
			.await?;
		Ok(())
	}

	// 子设备上线
	pub async fn login(&self, login_param: LoginParam) -> Result<()> {
		let payload = SubDevLoginRequest {
			id: global_id_next().to_string(),
			params: DeviceInfo::new(login_param.product_key, login_param.device_name, Some(login_param.clean_session)),
		};
		self.publish(format!("/ext/session/{}/{}/combine/login", self.three.product_key, self.three.device_name), &payload).await
	}

	// 子设备批量上线
	pub async fn batch_login(&self, login_params: &[LoginParam]) -> Result<()> {
		let payload = SubDevBatchLoginRequest {
			id: global_id_next().to_string(),
			params: SubDevBatchLoginParams {
				device_list: login_params.iter()
					.map(|n| DeviceInfo::new(n.product_key.clone(), n.device_name.clone(), Some(n.clean_session)))
					.collect()
			},
		};
		self.publish(format!("/ext/session/{}/{}/combine/batch_login", self.three.product_key, self.three.device_name), &payload).await
	}

	// 子设备下线
	pub async fn logout(&self, device_info: DeviceInfoId) -> Result<()> {
		let payload = SubDevLogoutRequest {
			id: global_id_next(),
			params: device_info,
		};
		self.publish(format!("/ext/session/{}/{}/combine/logout", self.three.product_key, self.three.device_name), &payload).await
	}

	// 子设备批量下线
	pub async fn batch_logout(&self, device_infos: &[DeviceInfoId]) -> Result<()> {
		let payload = SubDevBatchLogoutRequest {
			id: global_id_next(),
			params: device_infos.to_vec(),
		};
		self.publish(format!("/ext/session/{}/{}/combine/batch_logout", self.three.product_key, self.three.device_name), &payload).await
	}

	// 子设备禁用
	pub async fn disable(&self) -> Result<()> {
		let payload = SubDevMethodRequest {
			id: global_id_next().to_string(),
			version: String::from(ALINK_VERSION),
			method: String::from("thing.disable"),
		};
		self.publish(format!("sys/{}/{}/thing/disable", self.three.product_key, self.three.device_name), &payload).await
	}

	// 子设备启用
	pub async fn enable(&self) -> Result<()> {
		let payload = SubDevMethodRequest {
			id: global_id_next().to_string(),
			version: String::from(ALINK_VERSION),
			method: String::from("thing.enable"),
		};
		self.publish(format!("sys/{}/{}/thing/enable", self.three.product_key, self.three.device_name), &payload).await
	}

	// 子设备删除
	pub async fn delete(&self) -> Result<()> {
		let payload = SubDevMethodRequest {
			id: global_id_next().to_string(),
			version: String::from(ALINK_VERSION),
			method: String::from("thing.delete"),
		};
		self.publish(format!("sys/{}/{}/thing/delete", self.three.product_key, self.three.device_name), &payload).await
	}

	// 添加拓扑关系
	pub async fn add_topological_relation(&self, device_infos: &[DeviceInfoId], ack: bool) -> Result<()> {
		let payload = SubDevAddTopologicalRelationRequest {
			id: global_id_next().to_string(),
			version: String::from(ALINK_VERSION),
			params: device_infos.iter()
				.map(|n| DeviceInfo::new(n.product_key.clone(), n.device_name.clone(), None))
				.collect(),
			sys: SysAck {
				ack: ack.into()
			},
			method: String::from("thing.topo.add"),
		};
		self.publish(format!("/sys/{}/{}/thing/topo/add", self.three.product_key, self.three.device_name), &payload).await
	}

	// 删除拓扑关系
	pub async fn delete_topological_relation(&self, device_infos: &[DeviceInfoId], ack: bool) -> Result<()> {
		let payload = SubDevDeleteTopologicalRelationRequest {
			id: global_id_next().to_string(),
			version: String::from(ALINK_VERSION),
			params: device_infos.to_vec(),
			sys: SysAck {
				ack: ack.into()
			},
			method: String::from("thing.topo.delete"),
		};
		self.publish(format!("/sys/{}/{}/thing/topo/delete", self.three.product_key, self.three.device_name), &payload).await
	}

	// 获取拓扑关系
	pub async fn get_topological_relation(&self, ack: bool) -> Result<()> {
		let payload = SubDevGetTopologicalRelationRequest {
			id: global_id_next().to_string(),
			version: String::from(ALINK_VERSION),
			sys: SysAck {
				ack: ack.into()
			},
			method: String::from("thing.topo.get"),
		};
		self.publish(format!("/sys/{}/{}/thing/topo/get", self.three.product_key, self.three.device_name), &payload).await
	}

	// 发现设备信息上报
	pub async fn found_report(&self, device_infos: &[DeviceInfoId], ack: bool) -> Result<()> {
		let payload = SubDevFoundReportRequest {
			id: global_id_next().to_string(),
			version: String::from(ALINK_VERSION),
			params: device_infos.to_vec(),
			sys: SysAck {
				ack: ack.into()
			},
			method: String::from("thing.topo.get"),
		};
		self.publish(format!("/sys/{}/{}/thing/list/found", self.three.product_key, self.three.device_name), &payload).await
	}
}

pub struct Executor {
	three: Arc<ThreeTuple>,
	tx: Sender<recv::SubDevRecv>,
}

#[async_trait::async_trait]
impl crate::Executor for Executor {
	async fn execute(&self, topic: &str, payload: &[u8]) -> Result<()> {
		debug!("{} {}", topic, String::from_utf8_lossy(payload));
		// "/ota/device/upgrade/+/+",
		// if let Some(caps) = self.regs[0].captures(topic) {
		// 	debug!("upgrade validate product_key:{}, device_name:{}",caps[1].to_string(),caps[2].to_string());
		// 	if &caps[1] != self.three.product_key || &caps[2] != self.three.device_name {
		// 		return Ok(());
		// 	}
		// 	debug!("upgrade");
		// 	let payload: UpgradePackageRequest = serde_json::from_slice(&payload)?;
		// 	debug!("payload");
		// 	let data = recv::SubDevRecv {
		// 		device_name: caps[1].to_string(),
		// 		product_key: caps[2].to_string(),
		// 		data: RecvEnum::UpgradePackageRequest(payload),
		// 	};
		// 	debug!("upgrade send");
		// 	self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
		// 	debug!("upgrade send ok");
		// 	return Ok(());
		// }

		Ok(())
	}
}