#![allow(dead_code)]
#![allow(unused)]

//! Rust Link SDK，提供阿里云物联网平台的设备端 Rust 开发工具包（非官方）。
//!
//! This document won't be translated to English because "Aliyun IoT Platform" only has it's Chinese version.
//!
//! 遵循阿里云物联网平台定义的 [Alink 协议](https://help.aliyun.com/document_detail/90459.html)。

pub mod util;
pub use util::error::{Error, Result};

pub mod alink;
pub use alink::ThreeTuple;

pub mod mqtt;
pub use mqtt::{DeviceAuthInfo, MqttClient, MqttInstance};

pub mod http;
pub use http::Http;

pub mod ra;
pub use ra::RemoteAccess;

pub mod dm;
pub use dm::msg::{DataModelMsg, MsgEnum};
pub use dm::recv::{DataModelRecv, RecvEnum};
pub use dm::{DataModel, DataModelOptions};

pub mod dynregmq;
pub use dynregmq::{DynamicRegister, DynamicRegisterResult};

pub mod ntp;
pub use ntp::NtpService;

pub mod logpost;
pub mod ota;
pub mod alink_topic;
pub mod http_downloader;
pub mod subdev;
pub mod tag;
pub mod shadow;

pub use logpost::LogPost;

#[async_trait::async_trait]
pub(crate) trait Executor {
    async fn execute(&self, topic: &str, payload: &[u8]) -> Result<()>;
}
