//! 错误处理

use std::sync::PoisonError;

use crate::MqttClient;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ClientError(#[from] rumqttc::ClientError),
    #[error(transparent)]
    ConnectionError(#[from] rumqttc::ConnectionError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    WebSocketError(#[from] tungstenite::Error),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
    #[error(transparent)]
    RegexError(#[from] regex::Error),
    #[error(transparent)]
    SystemTimeError(#[from] std::time::SystemTimeError),
    #[error(transparent)]
    DownloadError(#[from] crate::http_downloader::Error),
    #[error(transparent)]
    HttpError(#[from] crate::https::HttpError),
    #[error("set system time error")]
    SetSystemTimeError,
    #[error("tokio mpsc send error")]
    MpscSendError,
    #[error("tokio oneshot recv error")]
    OneshotRecvError,
    #[error("tokio broadcast send error")]
    BroadcastSendError,
    #[error("header format error: {0}")]
    HeaderFormatError(String),
    #[error("session {0} not found")]
    SessionNotFound(String),
    #[error("send topic to proxy failed")]
    SendTopicError,
    #[error("recv topic from tx failed")]
    RecvTopicError,
    #[error("proxy upload data failed")]
    UploadDataError,
    #[error("keepalive timeout")]
    KeepaliveTimeout,
    #[error("HMAC can take key of any size")]
    CryptoInitError,
    #[error("add pem file failed")]
    AddPemFileError,
    #[error("invalid topic: {0}")]
    InvalidTopic(String),
    #[error("模块 {0} 不匹配")]
    TopicNotMatch(String),
    #[error("重复的注册响应")]
    RepeatRegisterResponse,
    #[error("事件循环错误")]
    EventLoopError,
    #[error("收取云端事件失败")]
    ReceiveCloudError,
    #[error("解析失败")]
    ParseTopicError,
    #[error("产品或设备名不匹配")]
    DeviceNameUnmatched,
    #[error("实例未初始化")]
    UnInitError,
    #[error("文件验证失败 {0}")]
    FileValidateFailed(String),
    #[error("Lock")]
    Lock,
    #[error("等待响应超时 {0}")]
    WaitResponseTimeout(String),
    #[error("无效路径")]
    InvalidPath,
    #[error("无效长度 {0} != {1}")]
    SizeNotMatch(usize, usize),
    #[error("错误码 {0} {1:?}")]
    CodeParams(u64, Option<String>),
    #[error("HTTP 请求构造失败")]
    HttpRequestBuild,
}
