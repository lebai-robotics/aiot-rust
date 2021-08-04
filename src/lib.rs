#![allow(dead_code)]

pub mod util;
pub use util::error::{Error, Result};

pub mod alink;

mod mqtt;
pub use mqtt::{DeviceAuthInfo, MqttClient, MqttInstance};

mod http;
pub use http::Http;

mod ra;
pub use ra::{RemoteAccess, RemoteAccessTrait, Runner as RemoteAccessRunner};

mod dm;
pub use dm::{DataModelOptions, DataModel, DataModelTrait};
pub use dm::msg::{DataModelMsg, MsgEnum};
pub use dm::recv::{DataModelRecv, RecvEnum};

mod dynregmq;
pub use dynregmq::{DynamicRegister, DynamicRegisterResult};

/// 设备证书三元组
#[derive(Debug, Clone, Default)]
pub struct ThreeTuple {
    // ProductKey
    pub product_key: String,
    // DeviceName
    pub device_name: String,
    // DeviceSecret
    pub device_secret: String,
}

impl ThreeTuple {
    pub fn from_env() -> Self {
        Self {
            product_key: std::env::var("PRODUCT_KEY").unwrap(),
            device_name: std::env::var("DEVICE_NAME").unwrap(),
            device_secret: std::env::var("DEVICE_SECRET").unwrap(),
        }
    }
}

#[async_trait::async_trait]
pub trait Executor {
    async fn execute(&self, topic: &str, payload: &[u8]) -> Result<()>;
}
