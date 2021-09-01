use crate::alink::{AlinkRequest, AlinkResponse};
use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReportVersion {
	pub version: String,
	pub module: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReportProgress {
	pub step: String,
	pub desc: String,
	pub module: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct QueryFirmware {
	pub module: String,
}


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OTAMsg {
	/// 消息所属设备的product_key, 若为NULL则使用通过aiot_dm_setopt配置的product_key
	/// 在网关子设备场景下, 可通过指定为子设备的product_key来发送子设备的消息到云端
	pub product_key: Option<String>,
	/// 消息所属设备的device_name, 若为NULL则使用通过aiot_dm_setopt配置的device_name
	/// 在网关子设备场景下, 可通过指定为子设备的product_key来发送子设备的消息到云端
	pub device_name: Option<String>,
	/// 消息数据
	pub data: MsgEnum,
}

/// data-model模块发送消息类型
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum MsgEnum {
	ReportVersion(ReportVersion),
	ReportProgress(ReportProgress),
	QueryFirmware(QueryFirmware),
}

impl OTAMsg {
	pub fn new(data: MsgEnum) -> Self {
		Self {
			product_key: None,
			device_name: None,
			data,
		}
	}

	pub fn to_payload(&self, ack: i32) -> Result<(String, Vec<u8>)> {
		let pk = self.product_key.as_deref().unwrap_or("");
		let dn = self.device_name.as_deref().unwrap_or("");
		self.data.to_payload(&pk, &dn, ack)
	}
}


impl OTAMsg {
	#[inline]
	pub fn report_version(report_version: ReportVersion) -> Self {
		OTAMsg::new(MsgEnum::ReportVersion(report_version))
	}
	#[inline]
	pub fn report_process(report_process: ReportProgress) -> Self {
		OTAMsg::new(MsgEnum::ReportProgress(report_process))
	}
	#[inline]
	pub fn query_firmware(query_firmware: QueryFirmware) -> Self {
		OTAMsg::new(MsgEnum::QueryFirmware(query_firmware))
	}
}

impl MsgEnum {
	pub fn to_payload(&self, pk: &str, dn: &str, ack: i32) -> Result<(String, Vec<u8>)> {
		use MsgEnum::*;
		match &self {
			ReportVersion(data) => {
				let topic = format!("/ota/device/inform/{}/{}", pk, dn);
				let payload = AlinkRequest::from_params(data);
				Ok((topic, serde_json::to_vec(&payload)?))
			}
			ReportProgress(data) => {
				let topic = format!("/ota/device/progress/{}/{}", pk, dn);
				let payload = AlinkRequest::from_params(data);
				Ok((topic, serde_json::to_vec(&payload)?))
			}
			QueryFirmware(data) => {
				let topic = format!("/sys/{}/{}/thing/ota/firmware/get", pk, dn);
				let payload = AlinkRequest {
					method: Some(String::from("thing.ota.firmware.get")),
					..AlinkRequest::from_params(data)
				};
				Ok((topic, serde_json::to_vec(&payload)?))
			}
		}
	}
}
