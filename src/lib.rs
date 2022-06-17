#![allow(dead_code)]
#![allow(unused)]

//! Rust Link SDK，提供阿里云物联网平台的设备端 Rust 开发工具包（非官方）。
//!
//! This document won't be translated to English because "Aliyun IoT Platform" only has it's Chinese version.
//!
//! 遵循阿里云物联网平台定义的 [Alink 协议](https://help.aliyun.com/document_detail/90459.html)。

pub use alink::aiot_module::{AiotModule, ModuleRecvKind};
pub use alink::ThreeTuple;
pub use dm::{DataModelMsg, DataModelOptions};
pub use dynregmq::{DynamicRegister, DynamicRegisterResult};
pub use http_downloader::HttpDownloader;
pub use https::Http;
pub use mqtt::{DeviceAuthInfo, MqttClient, MqttConnection, MqttInstance};
pub use ra::base::SecureTunnelNotify;
pub use tunnel::protocol::Service as LocalService;
pub use tunnel::proxy::{TunnelAction, TunnelParams, TunnelProxy};
pub use util::error::{Error, Result};

pub mod alink;
pub mod bootstrap;
pub mod dm;
pub mod dynregmq;
pub mod file;
pub mod http_downloader;
pub mod https;
pub mod logpost;
pub mod mqtt;
pub mod ntp;
pub mod ota;
pub mod ra;
pub mod remote_config;
pub mod shadow;
pub mod subdev;
pub mod tag;
pub mod tunnel;
pub mod util;

#[async_trait::async_trait]
pub trait Executor {
    async fn execute(&mut self, topic: &str, payload: &[u8]) -> Result<()>;
}

pub fn execute<RecvKind>(three: &ThreeTuple, topic: &str, payload: &[u8]) -> Result<RecvKind::Recv>
where
    RecvKind: ModuleRecvKind,
{
    log::debug!("receive: {} {}", topic, String::from_utf8_lossy(payload));
    if let Some((kind, caps)) = RecvKind::match_kind(topic, &three.product_key, &three.device_name)
    {
        kind.to_payload(payload, &caps)
    } else {
        Err(Error::TopicNotMatch(format!("{:?}", RecvKind::show())))
    }
}
