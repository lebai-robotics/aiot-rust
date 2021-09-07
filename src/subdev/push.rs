use crate::alink::{AlinkRequest, AlinkResponse};
use crate::alink::{global_id_next, SysAck, ALINK_VERSION};
use crate::subdev::base::*;
use crate::subdev::push_dto::*;

#[derive(Clone)]
pub struct LoginParam {
	pub product_key: String,
	pub device_name: String,
	pub device_secret: String,
	pub clean_session: bool,
}

impl crate::subdev::Runner {

	// 通知网关添加设备拓扑关系
	pub async fn notify_add_topological_relation(&self, id: String, code: u64) -> crate::Result<()> {
		let payload = AlinkResponse {
			id,
			code,
			method: None,
			version: None,
			data: (),
			message: None,
		};
		self
			.publish(
				format!(
					"/sys/{}/{}/thing/topo/add/notify_reply",
					self.three.product_key, self.three.device_name
				),
				&payload,
			)
			.await
	}
	// 禁用设备回应
	pub async fn disable_reply(&self, id: String, code: u64) -> crate::Result<()> {
		let payload = AlinkResponse {
			id,
			code,
			data: (),
			message: None,
			method: None,
			version: None,
		};
		self
			.publish(
				format!(
					"/sys/{}/{}/thing/disable_reply",
					self.three.product_key, self.three.device_name
				),
				&payload,
			)
			.await
	}

	// 启用设备回应
	pub async fn enable_reply(&self, id: String, code: u64) -> crate::Result<()> {
		let payload = AlinkResponse {
			id,
			code,
			data: (),
			message: None,
			method: None,
			version: None,
		};
		self
			.publish(
				format!(
					"/sys/{}/{}/thing/enable_reply",
					self.three.product_key, self.three.device_name
				),
				&payload,
			)
			.await
	}

	// 删除设备回应
	pub async fn delete_reply(&self, id: String, code: u64) -> crate::Result<()> {
		let payload = AlinkResponse {
			id,
			code,
			data: (),
			message: None,
			method: None,
			version: None,
		};
		self
			.publish(
				format!(
					"/sys/{}/{}/thing/delete_reply",
					self.three.product_key, self.three.device_name
				),
				&payload,
			)
			.await
	}
	// 子设备上线
	pub async fn login(&self, login_param: LoginParam) -> crate::Result<()> {
		let payload = SubDevLoginRequest {
			id: global_id_next().to_string(),
			params: DeviceInfo::new(
				login_param.product_key,
				login_param.device_name,
				Some(login_param.clean_session),
				login_param.device_secret,None
			),
			version: ALINK_VERSION.to_string(),
			sys: None,
			method: None,
		};
		self
			.publish(
				format!(
					"/ext/session/{}/{}/combine/login",
					self.three.product_key, self.three.device_name
				),
				&payload,
			)
			.await
	}

	// 子设备批量上线
	pub async fn batch_login(&self, login_params: &[LoginParam]) -> crate::Result<()> {
		let payload = SubDevBatchLoginRequest {
			id: global_id_next().to_string(),
			params: SubDevBatchLoginParams {
				device_list: login_params
					.iter()
					.map(|n| {
						DeviceInfo::new(
							n.product_key.clone(),
							n.device_name.clone(),
							Some(n.clean_session),
							n.device_secret.clone(),
							None
						)
					})
					.collect(),
			},
			version: ALINK_VERSION.to_string(),
			sys: None,
			method: None,
		};
		self
			.publish(
				format!(
					"/ext/session/{}/{}/combine/batch_login",
					self.three.product_key, self.three.device_name
				),
				&payload,
			)
			.await
	}

	// 子设备下线
	pub async fn logout(&self, device_info: DeviceInfoId) -> crate::Result<()> {
		let payload = SubDevLogoutRequest {
			id: global_id_next().to_string(),
			params: device_info,
			version: ALINK_VERSION.to_string(),
			sys: None,
			method: None,
		};
		self
			.publish(
				format!(
					"/ext/session/{}/{}/combine/logout",
					self.three.product_key, self.three.device_name
				),
				&payload,
			)
			.await
	}

	// 子设备批量下线
	pub async fn batch_logout(&self, device_infos: &[DeviceInfoId]) -> crate::Result<()> {
		let payload = SubDevBatchLogoutRequest {
			id: global_id_next().to_string(),
			params: device_infos.to_vec(),
			version: ALINK_VERSION.to_string(),
			sys: None,
			method: None,
		};
		self
			.publish(
				format!(
					"/ext/session/{}/{}/combine/batch_logout",
					self.three.product_key, self.three.device_name
				),
				&payload,
			)
			.await
	}

	// 添加拓扑关系
	pub async fn add_topological_relation(
		&self,
		device_infos: &[DeviceInfoWithSecret],
		ack: bool,
	) -> crate::Result<()> {
		let payload = SubDevAddTopologicalRelationRequest {
			id: global_id_next().to_string(),
			version: String::from(ALINK_VERSION),
			params: device_infos
				.iter()
				.map(|n| {
					DeviceInfo::new(
						n.product_key.clone(),
						n.device_name.clone(),
						None,
						n.device_secret.clone(),
						None
					)
				})
				.collect::<Vec<DeviceInfo>>(),
			sys: Some(SysAck { ack: ack.into() }),
			method: Some(String::from("thing.topo.add")),
		};
		self
			.publish(
				format!(
					"/sys/{}/{}/thing/topo/add",
					self.three.product_key, self.three.device_name
				),
				&payload,
			)
			.await
	}

	// 删除拓扑关系
	pub async fn delete_topological_relation(
		&self,
		device_infos: &[DeviceInfoId],
		ack: bool,
	) -> crate::Result<()> {
		let payload = SubDevDeleteTopologicalRelationRequest {
			id: global_id_next().to_string(),
			version: String::from(ALINK_VERSION),
			params: device_infos.to_vec(),
			sys: Some(SysAck { ack: ack.into() }),
			method: Some(String::from("thing.topo.delete")),
		};
		self
			.publish(
				format!(
					"/sys/{}/{}/thing/topo/delete",
					self.three.product_key, self.three.device_name
				),
				&payload,
			)
			.await
	}

	// 获取拓扑关系
	pub async fn get_topological_relation(&self, ack: bool) -> crate::Result<()> {
		let payload = SubDevGetTopologicalRelationRequest {
			id: global_id_next().to_string(),
			version: String::from(ALINK_VERSION),
			sys: Some(SysAck { ack: ack.into() }),
			method: Some(String::from("thing.topo.get")),
			params: None,
		};
		self
			.publish(
				format!(
					"/sys/{}/{}/thing/topo/get",
					self.three.product_key, self.three.device_name
				),
				&payload,
			)
			.await
	}

	// 发现设备信息上报
	pub async fn found_report(&self, device_infos: &[DeviceInfoId], ack: bool) -> crate::Result<()> {
		let payload = SubDevFoundReportRequest {
			id: global_id_next().to_string(),
			version: String::from(ALINK_VERSION),
			params: device_infos.to_vec(),
			sys: Some(SysAck { ack: ack.into() }),
			method: Some(String::from("thing.topo.get")),
		};
		self
			.publish(
				format!(
					"/sys/{}/{}/thing/list/found",
					self.three.product_key, self.three.device_name
				),
				&payload,
			)
			.await
	}


}
