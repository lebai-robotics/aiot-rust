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
