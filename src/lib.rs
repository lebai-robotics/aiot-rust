#![allow(dead_code)]

//! # Rust Link SDK
//!
//! 提供阿里云物联网平台的设备端 Rust 开发工具包（非阿里官方）。
//!
//! This document won't be translated to English because "Aliyun IoT Platform" only has it's Chinese version.
//!
//! 阿里官方的 [Link SDK](https://help.aliyun.com/document_detail/96596.html) 提供了以下语言的版本：
//! - [C Link SDK](https://help.aliyun.com/document_detail/163753.html)
//! - [Android Link SDK](https://help.aliyun.com/document_detail/96605.html)
//! - [Node.js Link SDK](https://help.aliyun.com/document_detail/96617.html)
//! - [Java Link SDK](https://help.aliyun.com/document_detail/97330.html)
//! - [Python Link SDK](https://help.aliyun.com/document_detail/98291.html)
//! - [iOS Link SDK](https://help.aliyun.com/document_detail/100532.html)
//!
//! 其中，C Link SDK 是功能最完整的，我们的 Rust SDK 也是对标这个进行设计开发的。
//! 刚开始使用 Rust 时候，尝试基于 C SDK 在 [`std::ffi`] 基础上进行封装，发现这种方式性能和可扩展性都不高，于是基于 [`rumqttc`] 和 [`tokio`] 实现了现在的版本。
//!
//! 本 crate 遵循阿里云物联网平台定义的 [Alink 协议](https://help.aliyun.com/document_detail/90459.html)。

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
pub use logpost::LogPost;

#[async_trait::async_trait]
pub(crate) trait Executor {
    async fn execute(&self, topic: &str, payload: &[u8]) -> Result<()>;
}
