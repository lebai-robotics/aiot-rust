use serde::Serialize;

/*pub struct AlinkTopic {
	pub topic: &'static str,
}

impl AlinkTopic {
	pub const fn new(topic: &'static str) -> Self {
		Self {
			topic
		}
	}
}

// 固件升级包信息
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PackageData {
	pub size: u32,
	pub version: String,
	pub is_diff: Option<bool>,
	pub url: String,
	pub md5: String,
	pub sign: String,
	pub sign_method: String,
	pub module: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UpgradePackageRequest {
	pub code: String,
	pub data: PackageData,
	pub id: u64,
	pub message: String,
}

pub trait MqttRequest: ?Sized + Serialize {}

pub mod ota_topics {
	use crate::alink_topic::AlinkTopic;

	pub static ss: AlinkTopic = AlinkTopic::new("xx");
}*/