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
	pub module: Option<String>,
}
pub type ReportVersionRequest = AlinkRequest<ReportVersion>;
pub type ReportProgressRequest = AlinkRequest<ReportProgress>;
pub type QueryFirmwareRequest = AlinkRequest<QueryFirmware>;

