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
#[serde(rename_all = "camelCase")]
pub struct LogPostRequest {
    /// 日志的采集时间，为设备本地UTC时间，包含时区信息，以毫秒计，格式为`yyyy-MM-dd'T'HH:mm:ss.SSSZ`。 可上报其它字符串格式，但不利于问题排查，不推荐使用。
    pub utc_time: String,
    /// 日志级别。可以使用默认日志级别，也可以自定义日志级别。默认日志级别从高到低为：
    /// - FATAL
    /// - ERROR
    /// - WARN
    /// - INFO
    /// - DEBUG
    pub log_level: String,
    /// 模块名称：
    /// - 当设备端使用Android SDK时，模块名称为ALK-LK。
    /// - 当设备端使用C SDK时，需自定义模块名称。
    /// - 当设备端使用自行开发的SDK时，需自定义模块名称。
    pub module: String,
    /// 结果状态码：
    /// - 当设备端使用Android SDK时，请参见错误码。
    /// - 当设备端使用C SDK时，请参见C SDK状态码。
    /// - 当设备端使用自行开发的SDK时，可以自定义结果状态码，也可以为空。
    pub code: String,
    /// 可选参数，上下文跟踪内容，设备端使用Alink协议消息的`id`，App端使用`TraceId`（追踪ID）。
    pub trace_context: Option<String>,
    /// 日志内容详情。
    pub log_content: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LogConfigContent {
    /// 设备日志上报模式，0表示设备SDK不上报日志，1表示设备SDK上报日志。
    pub mode: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LogConfig {
    /// 获取内容类型，默认为`content`。因日志配置内容较少，默认直接返回内容。
    pub get_type: String,
    /// 配置文本内容。
    pub content: LogConfigContent,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LogConfigGet {
    /// 配置范围，目前日志只有设备维度配置，默认为device。
    pub config_scope: String,
    /// 获取内容类型，默认为`content`。因日志配置内容较少，默认直接返回内容。
    pub get_type: String,
}
