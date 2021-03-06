//! ALink 基础协议。

use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;

pub mod aiot_module;
pub mod alink_topic;

/// 设备认证三元组。
///
/// 这个三元组除了在一型一密的动态注册中用不到之外，所有和阿里云物联网平台发起的连接都要有这三元组生成签名。
#[derive(Debug, Clone, Default)]
pub struct ThreeTuple {
    /// 产品 ProductKey，即所谓”型“。参考[创建产品](https://help.aliyun.com/document_detail/73728.html)。
    pub product_key: String,
    /// 设备 DeviceName，即所谓”机“。参考[创建设备](https://help.aliyun.com/document_detail/89271.html)。
    pub device_name: String,
    /// 设备 DeviceSecret，即所谓”密“。参考[设备安全认证](https://help.aliyun.com/document_detail/74004.html)和[设备获取设备证书](https://help.aliyun.com/document_detail/157846.html)。
    pub device_secret: String,
}

impl ThreeTuple {
    /// 从下列环境变量中读取三元组。
    ///
    /// - `PRODUCT_KEY`
    /// - `DEVICE_NAME`
    /// - `DEVICE_SECRET`
    ///
    /// 这个方法在示例代码中多处用到，但实际生产环境中建议自行编写初始化逻辑。
    ///
    /// # Examples
    ///
    /// ```
    /// std::env::set_var("PRODUCT_KEY", "a");
    /// std::env::set_var("DEVICE_NAME", "b");
    /// std::env::set_var("DEVICE_SECRET", "c");
    ///
    /// use aiot::ThreeTuple;
    ///
    /// let three = ThreeTuple::from_env();
    /// assert_eq!("a", &three.product_key);
    /// assert_eq!("b", &three.device_name);
    /// assert_eq!("c", &three.device_secret);
    /// ```
    ///
    /// # Panics
    ///
    /// 如果没有设置对应的环境变量，该方法将 panic。
    pub fn from_env() -> Self {
        Self {
            product_key: std::env::var("PRODUCT_KEY").unwrap(),
            device_name: std::env::var("DEVICE_NAME").unwrap(),
            device_secret: std::env::var("DEVICE_SECRET").unwrap(),
        }
    }
}

static ID: AtomicU64 = AtomicU64::new(1);

pub fn global_id_next() -> u64 {
    ID.fetch_add(1, Ordering::SeqCst)
}

#[serde_as]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SimpleResponse {
    pub id: String,
    #[serde_as(as = "_")]
    pub code: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ParamsRequest<T> {
    /// 消息ID号。String类型的数字，取值范围0~4294967295，且每个消息ID在当前设备中具有唯一性。
    pub id: String,
    /// 请求业务参数。
    pub params: T,
}

impl<T> ParamsRequest<T> {
    pub fn new(params: T) -> Self {
        Self {
            id: global_id_next().to_string(),
            params,
        }
    }
}

#[serde_as]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AlinkResponse<T = Option<()>, TID = String, TCode = u64> {
    pub id: TID,
    // #[serde_as(as = "u64")]
    // #[serde_as(as = "DisplayFromStr")]
    pub code: TCode,
    pub data: T,
    pub message: Option<String>,
    pub method: Option<String>,
    pub version: Option<String>,
}

impl<T> AlinkResponse<T> {
    pub fn msg_id(&self) -> u64 {
        self.id.parse().unwrap_or(0)
    }

    pub fn new(id: u64, code: u64, data: T) -> Self {
        Self {
            id: format!("{}", id),
            code,
            data,
            message: None,
            version: None,
            method: None,
        }
    }
}

pub const ALINK_VERSION: &str = "1.0";

#[serde_as]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AlinkRequest<T = Option<()>> {
    // #[serde_as(as = "String")]
    pub id: String,
    pub version: String,
    pub params: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sys: Option<SysAck>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
}

impl<T> AlinkRequest<T> {
    pub fn msg_id(&self) -> u64 {
        self.id.parse().unwrap_or(0)
    }

    pub fn new_id(id: u64, method: &str, params: T, ack: Option<i32>) -> Self {
        Self {
            id: format!("{}", id),
            version: ALINK_VERSION.to_string(),
            params,
            sys: ack.map(|ack| SysAck { ack }),
            method: Some(method.to_string()),
        }
    }

    pub fn from_params(params: T) -> Self {
        Self {
            id: global_id_next().to_string(),
            version: ALINK_VERSION.to_string(),
            params,
            sys: None,
            method: None,
        }
    }
    pub fn new(method: &str, params: T, ack: i32) -> Self {
        Self::new_id(global_id_next(), method, params, Some(ack))
    }
    pub fn new_no_ack(method: &str, params: T) -> Self {
        Self::new_id(global_id_next(), method, params, None)
    }
}

/// sys下的扩展功能字段，表示是否返回响应数据。1：云端返回响应数据。0：云端不返回响应数据。
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SysAck {
    pub ack: i32,
}
