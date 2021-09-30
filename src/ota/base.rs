use serde_json::Value;
use std::collections::HashMap;

use crate::alink::alink_topic::ALinkSubscribeTopic;
use crate::alink::{AlinkRequest, AlinkResponse};
use crate::subdev::base::DeviceInfoId;
use enum_iterator::IntoEnumIterator;
use enum_kinds::EnumKind;
use serde::{Deserialize, Serialize};

/// 固件升级包信息
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PackageData {
    /// 大小
    pub size: u64,
    /// 版本
    pub version: String,
    /// 是否使用了差分升级
    pub is_diff: Option<u8>,
    /// 包Url
    pub url: String,
    /// MD5
    pub md5: Option<String>,
    /// 签名
    pub sign: String,
    /// 签名方法
    pub sign_method: String,
    /// 升级包所属模块名
    pub module: Option<String>,
    /// 升级批次标签列表和推送给设备的自定义信息。
    /// _package_udi表示自定义信息的字段。
    /// 单个标签格式："key":"value"。
    pub ext_data: Option<Value>,
}
