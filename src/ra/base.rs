use crate::alink::{AlinkRequest, AlinkResponse};
use crate::{Error, Result, ThreeTuple};
use log::*;
use regex::Regex;
use rumqttc::{AsyncClient, QoS};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct EdgeDebugSwitch {
    pub status: i32,
}

impl EdgeDebugSwitch {
    pub fn is_open(&self) -> bool {
        self.status != 0
    }
}

#[derive(Debug, Clone)]
pub struct RemoteAccessOptions {
    pub three: Arc<ThreeTuple>,
    pub cloud_host: String,
    //远程连接通道云端服务地址，可以是域名
    pub cloud_port: u32, //远程连接通道云端服务端口
}

impl RemoteAccessOptions {
    pub fn new(three: Arc<ThreeTuple>) -> Self {
        Self {
            three,
            cloud_host: "backend-iotx-remote-debug.aliyun.com".to_string(),
            cloud_port: 443,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged, rename_all = "snake_case")]
/// operation
pub enum SecureTunnelNotify {
    Connect(SecureTunnelConnect),
    Close(SecureTunnelClose),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SecureTunnelConnect {
    /// SSH远程通道的ID。
    pub tunnel_id: String,
    pub path: String,
    /// 设备与SSH远程通道进行WebSocket建连的认证Token。
    /// **说明** 新创建的Token有效期为7天。设备端请求SSH通道建连信息时，若物联网平台发现Token的生成时间超过了5天，则更新建连信息后响应设备的请求。
    pub session_id: String,
    /// 设备与SSH远程通道进行WebSocket建连的URL。
    pub uri: String,
    /// uri的域名。
    pub host: String,
    /// uri的端口号。
    pub port: u16,
    /// session_id剩余的有效时间，单位为秒。
    /// **说明**
    /// - session_id过期后，设备会收到关闭通知，设备需主动关闭该SSH通道。
    ///   若session_id过期后，设备未主动关闭SSH通道，物联网平台会在session_id过期后5秒后主动关闭SSH通道。
    /// - 设备端需要周期性主动请求更新建连信息，并使用新的建连信息重连SSH通道，避免云端监测到session_id过期时，下发关闭通知及主动关闭通道。
    /// - session_id过期前或过期时，您也可手动关闭远程登录功能，使设备与SSH远程通道断连。具体操作，请参见关闭远程登录。
    ///   关闭操作仅对当前的SSH通道连接有效，不会禁止设备端后续的SSH通道建连行为。若设备端再次请求建连信息后仍可以成功进行SSH通道建连。
    pub session_id_expire: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SecureTunnelClose {
    // pub operation: String,
    pub close_reason: String,
}
