use serde_json::Value;

// 影子设备更新
// /shadow/update/${YourProductKey}/${YourDeviceName}
pub struct ShadowUpdateRequest {
	method: String,
	state: Value,
	version: u32,
}

// 影子设备获取
// /shadow/get/${YourProductKey}/${YourDeviceName}
pub struct ShadowGetTopic {
	method: String,
	payload: Value,
	version: u32,
	timestamp: u64,
}