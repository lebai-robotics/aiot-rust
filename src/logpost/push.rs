use super::base::*;
use crate::alink::{AlinkRequest, AlinkResponse, SysAck};
use crate::{Error, Result, ThreeTuple};
use regex::Regex;
use rumqttc::{AsyncClient, QoS};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

impl super::Module {
    /// 设备获取日志配置
    ///
    /// # 参数
    ///
    /// * `config_scope` - 配置范围，目前日志只有设备维度配置，默认为device。
    /// * `get_type` - 获取内容类型，默认为content。因日志配置内容较少，默认直接返回内容。
    pub async fn get(&self, config_scope: &str, get_type: &str) -> crate::Result<()> {
        let payload: LogConfigGetRequest = AlinkRequest::new(
            "thing.config.log.get",
            LogConfigGet {
                config_scope: config_scope.to_string(),
                get_type: get_type.to_string(),
            },
            1,
        );
        let topic = format!(
            "/sys/{}/{}/thing/config/log/get",
            self.three.product_key, self.three.device_name
        );
        self.publish(topic, &payload).await
    }

    pub async fn get_default(&self) -> crate::Result<()> {
        self.get("device", "content").await
    }

    pub async fn post(&self, logs: Vec<LogItem>) -> crate::Result<()> {
        let payload: LogPostRequest = AlinkRequest::new("thing.log.post", logs, 1);
        let topic = format!(
            "/sys/{}/{}/thing/config/log/get",
            self.three.product_key, self.three.device_name
        );
        self.publish(topic, &payload).await
    }
}

/// 设备获取日志配置
pub type LogConfigGetRequest = AlinkRequest<LogConfigGet>;

/// 设备上报日志内容
/// 数组元素最多40个。
pub type LogPostRequest = AlinkRequest<Vec<LogItem>>;
