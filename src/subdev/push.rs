use crate::alink::{SysAck, ALINK_VERSION, global_id_next};
use crate::subdev::push_dto::*;
use crate::subdev::base::*;


pub struct LoginParam {
	pub product_key: String,
	pub device_name: String,
	pub clean_session: bool,
}

impl crate::subdev::Runner {
	// 子设备上线
	pub async fn login(&self, login_param: LoginParam) -> crate::Result<()> {
		let payload = SubDevLoginRequest {
			id: global_id_next().to_string(),
			params: DeviceInfo::new(login_param.product_key, login_param.device_name, Some(login_param.clean_session)),
		};
		self.publish(format!("/ext/session/{}/{}/combine/login", self.three.product_key, self.three.device_name), &payload).await
	}

	// 子设备批量上线
	pub async fn batch_login(&self, login_params: &[LoginParam]) -> crate::Result<()> {
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
	pub async fn logout(&self, device_info: DeviceInfoId) -> crate::Result<()> {
		let payload = SubDevLogoutRequest {
			id: global_id_next(),
			params: device_info,
		};
		self.publish(format!("/ext/session/{}/{}/combine/logout", self.three.product_key, self.three.device_name), &payload).await
	}

	// 子设备批量下线
	pub async fn batch_logout(&self, device_infos: &[DeviceInfoId]) -> crate::Result<()> {
		let payload = SubDevBatchLogoutRequest {
			id: global_id_next(),
			params: device_infos.to_vec(),
		};
		self.publish(format!("/ext/session/{}/{}/combine/batch_logout", self.three.product_key, self.three.device_name), &payload).await
	}

	// 子设备禁用
	pub async fn disable(&self) -> crate::Result<()> {
		let payload = SubDevMethodRequest {
			id: global_id_next().to_string(),
			version: String::from(ALINK_VERSION),
			method: String::from("thing.disable"),
		};
		self.publish(format!("sys/{}/{}/thing/disable", self.three.product_key, self.three.device_name), &payload).await
	}

	// 子设备启用
	pub async fn enable(&self) -> crate::Result<()> {
		let payload = SubDevMethodRequest {
			id: global_id_next().to_string(),
			version: String::from(ALINK_VERSION),
			method: String::from("thing.enable"),
		};
		self.publish(format!("sys/{}/{}/thing/enable", self.three.product_key, self.three.device_name), &payload).await
	}

	// 子设备删除
	pub async fn delete(&self) -> crate::Result<()> {
		let payload = SubDevMethodRequest {
			id: global_id_next().to_string(),
			version: String::from(ALINK_VERSION),
			method: String::from("thing.delete"),
		};
		self.publish(format!("sys/{}/{}/thing/delete", self.three.product_key, self.three.device_name), &payload).await
	}

	// 添加拓扑关系
	pub async fn add_topological_relation(&self, device_infos: &[DeviceInfoId], ack: bool) -> crate::Result<()> {
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
	pub async fn delete_topological_relation(&self, device_infos: &[DeviceInfoId], ack: bool) -> crate::Result<()> {
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
	pub async fn get_topological_relation(&self, ack: bool) -> crate::Result<()> {
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
	pub async fn found_report(&self, device_infos: &[DeviceInfoId], ack: bool) -> crate::Result<()> {
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