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
    #[error("tokio mpsc send error")]
    MpscSendError,
    #[error("header format error: {0}")]
    HeaderFormatError(String),
    #[error("session {0} not found")]
    SessionNotFound(String),
    #[error("send topic to proxy failed")]
    SendTopicError,
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
    #[error("重复的注册响应")]
    RepeatRegisterResponse,
    #[error("事件循环错误")]
    EventLoopError,
    #[error("收取云端事件失败")]
    ReceiveCloudError,
}
