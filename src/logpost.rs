//! 日志上报。

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
pub struct LogPostRequest {
    /// 日志的采集时间，为设备本地UTC时间，包含时区信息，以毫秒计，格式为`yyyy-MM-dd'T'HH:mm:ss.SSSZ`。 可上报其它字符串格式，但不利于问题排查，不推荐使用。
    #[serde(rename = "utcTime")]
    pub utc_time: String,
    /// 日志级别。可以使用默认日志级别，也可以自定义日志级别。默认日志级别从高到低为：
    /// - FATAL
    /// - ERROR
    /// - WARN
    /// - INFO
    /// - DEBUG
    #[serde(rename = "logLevel")]
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
    #[serde(rename = "traceContext")]
    /// 可选参数，上下文跟踪内容，设备端使用Alink协议消息的`id`，App端使用`TraceId`（追踪ID）。
    pub trace_context: Option<String>,
    /// 日志内容详情。
    #[serde(rename = "logContent")]
    pub log_content: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LogConfigContent {
    /// 设备日志上报模式，0表示设备SDK不上报日志，1表示设备SDK上报日志。
    pub mode: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LogConfig {
    /// 获取内容类型，默认为`content`。因日志配置内容较少，默认直接返回内容。
    #[serde(rename = "getType")]
    pub get_type: String,
    /// 配置文本内容。
    pub content: LogConfigContent,
}

pub struct HalfRunner {
    post_topic: String,
    config_topic: String,
    rx: Receiver<LogConfig>,
}

impl HalfRunner {
    pub async fn init(self, client: &AsyncClient) -> Result<Runner> {
        client
            .subscribe("/sys/+/+/thing/config/log/push", QoS::AtLeastOnce)
            .await?;
        client
            .subscribe("/sys/+/+/thing/config/log/get_reply", QoS::AtLeastOnce)
            .await?;
        Ok(Runner {
            post_topic: self.post_topic,
            config_topic: self.config_topic,
            rx: self.rx,
            client: client.clone(),
        })
    }
}

pub struct Runner {
    post_topic: String,
    config_topic: String,
    rx: Receiver<LogConfig>,
    client: AsyncClient,
}

impl Runner {
    // pub async fn send(&self) -> Result<()> {
    //     let data = LogPostRequest {
    //         device_send_time: now.as_millis() as u64,
    //     };
    //     let data = serde_json::to_string(&data)?;
    //     self.client
    //         .publish(&self.topic, QoS::AtMostOnce, false, data)
    //         .await?;
    //     Ok(())
    // }

    pub async fn poll(&mut self) -> Result<LogConfig> {
        let data = self.rx.recv().await.ok_or(Error::ReceiveCloudError)?;
        debug!("{:?}", data);
        Ok(data)
    }
}

pub struct Executor {
    three: Arc<ThreeTuple>,
    get_reply: Regex,
    push: Regex,
    tx: Sender<LogConfig>,
}

#[async_trait::async_trait]
impl crate::Executor for Executor {
    async fn execute(&self, topic: &str, payload: &[u8]) -> Result<()> {
        let data = if let Some(caps) = self.get_reply.captures(topic) {
            if &caps[1] == &self.three.product_key && &caps[2] == &self.three.device_name {
                let data: AlinkResponse<LogConfig> = serde_json::from_slice(payload)?;
                Ok(data.data)
            } else {
                Err(Error::DeviceNameUnmatched)
            }
        } else if let Some(caps) = self.push.captures(topic) {
            if &caps[1] == &self.three.product_key && &caps[2] == &self.three.device_name {
                let data: AlinkRequest<LogConfig> = serde_json::from_slice(payload)?;
                Ok(data.params)
            } else {
                Err(Error::DeviceNameUnmatched)
            }
        } else {
            Err(Error::ParseTopicError)
        };
        self.tx.send(data?).await.map_err(|_| Error::SendTopicError)
    }
}

struct LogPostInner {
    runner: HalfRunner,
    executor: Executor,
}

impl LogPostInner {
    fn new(three: Arc<ThreeTuple>) -> Result<Self> {
        let (tx, rx) = mpsc::channel(16);

        let get_reply = Regex::new(r"/sys/(.*)/(.*)/thing/config/log/get_reply")?;
        let push = Regex::new(r"/sys/(.*)/(.*)/thing/config/log/push")?;
        let executor = Executor {
            three: three.clone(),
            get_reply,
            push,
            tx,
        };

        let post_topic = format!(
            "/sys/{}/{}/thing/log/post",
            three.product_key, three.device_name
        );
        let config_topic = format!(
            "/sys/{}/{}/thing/config/log/get",
            three.product_key, three.device_name
        );
        let runner = HalfRunner {
            post_topic,
            config_topic,
            rx,
        };

        Ok(Self { runner, executor })
    }
}

pub trait LogPost {
    fn log_post(&mut self) -> Result<HalfRunner>;
}

impl LogPost for crate::MqttClient {
    fn log_post(&mut self) -> Result<HalfRunner> {
        let ra = LogPostInner::new(self.three.clone())?;
        self.executors
            .push(Box::new(ra.executor) as Box<dyn crate::Executor>);
        Ok(ra.runner)
    }
}
