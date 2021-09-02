use crate::alink::{AlinkRequest, AlinkResponse};

pub struct DeviceInfoKeyValue {
	attr_key: String,
	attr_value: String,
}

pub struct DeviceInfoKey {
	attr_key: String,
}

// 标签信息上报
// /sys/{productKey}/{deviceName}/thing/deviceinfo/update
pub type DeviceInfoUpdateRequest = AlinkRequest<Vec<DeviceInfoKeyValue>>;

// 标签信息上报响应
// /sys/{productKey}/{deviceName}/thing/deviceinfo/update_reply
pub type DeviceInfoUpdateResponse = AlinkResponse<()>;

// 标签信息删除
// /sys/{productKey}/{deviceName}/thing/deviceinfo/delete
pub type DeviceInfoDeleteRequest = AlinkRequest<Vec<DeviceInfoKey>>;

// 标签信息删除响应
// /sys/{productKey}/{deviceName}/thing/deviceinfo/delete_replly
pub type DeviceInfoDeleteResponse = AlinkResponse<()>;