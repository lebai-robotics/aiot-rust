use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_repr::{Deserialize_repr, Serialize_repr};

/// 本地服务信息
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Service {
    /// 服务类型
    pub r#type: String,
    /// 服务IP地址/host
    pub ip: String,
    /// 服务端口号
    pub port: u16,
}

impl Default for Service {
    fn default() -> Self {
        Self {
            r#type: "_SSH".to_string(),
            ip: "127.0.0.1".to_string(),
            port: 22,
        }
    }
}

/// 消息报文头格式-->通用
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Header {
    /// 隧道帧类型。可
    pub frame_type: FrameType,
    /// 访问端或设备端发送通信数据时设置的帧ID，取值范围为0~（263-1）。
    /// 建议设备端和访问端均使用递增的帧ID，用于区分每个session_id会话中的通信数据。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_id: Option<u64>,
    /// 不同类型隧道帧的会话ID，在当前安全隧道内唯一。
    /// 访问端发送创建Session的请求帧时，不需要传入该参数，物联网平台会根据收到的请求帧分配一个会话ID，并发送给设备端。其他类型的隧道帧，访问端和设备端均需要传递会话ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Session对应的业务类型，由您自定义。支持英文字母、下划线（_）、短划线（-）和英文句号（.），首字母必须为英文字母，最长不超过16个字符。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_type: Option<String>,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Copy, Clone, PartialEq)]
#[repr(u16)]
pub enum FrameType {
    /// common_response，响应数据。
    Response = 1,
    /// 新增session
    NewSession = 2,
    /// 释放session
    ReleaseSession = 3,
    /// 服务发送的原始服务协议.
    RawData = 4,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ResponseBody<T> {
    pub code: T,
    pub msg: String,
}

#[derive(Debug)]
pub struct Frame {
    pub header: Header,
    pub body: Vec<u8>,
}

impl Frame {
    pub fn new(header: Header, body: Vec<u8>) -> Frame {
        Frame { header, body }
    }

    pub fn new_session(frame_id: u64, service_type: String) -> Frame {
        Frame::new(
            Header {
                frame_type: FrameType::NewSession,
                frame_id: Some(frame_id),
                session_id: None,
                service_type: Some(service_type),
            },
            Vec::new(),
        )
    }

    pub fn raw(session_id: String, frame_id: u64, service_type: String, body: Vec<u8>) -> Frame {
        Frame::new(
            Header {
                frame_type: FrameType::RawData,
                frame_id: Some(frame_id),
                session_id: Some(session_id),
                service_type: Some(service_type),
            },
            body,
        )
    }

    pub fn response(
        session_id: String,
        frame_id: u64,
        service_type: String,
        code: u8,
        msg: String,
    ) -> Frame {
        Frame::new(
            Header {
                frame_type: FrameType::Response,
                frame_id: Some(frame_id),
                session_id: Some(session_id),
                service_type: Some(service_type),
            },
            serde_json::to_vec(&ResponseBody { code, msg }).unwrap(),
        )
    }

    pub fn release(session_id: String, frame_id: u64, code: u8, msg: String) -> Frame {
        Frame::new(
            Header {
                frame_type: FrameType::ReleaseSession,
                frame_id: Some(frame_id),
                session_id: Some(session_id),
                service_type: None,
            },
            serde_json::to_vec(&ResponseBody { code, msg }).unwrap(),
        )
    }

    pub fn to_vec(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        let header = serde_json::to_vec(&self.header).map_err(Error::SerdeError)?;
        buf.extend_from_slice(&(header.len() as u16).to_be_bytes());
        buf.extend_from_slice(&header);
        buf.extend_from_slice(&self.body);
        Ok(buf)
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 2 {
            return Err(Error::HeaderFormatError("长度不够".into()));
        }
        let len = u16::from_be_bytes(bytes[..2].try_into().unwrap()) as usize;
        if bytes.len() < 2 + len {
            return Err(Error::HeaderFormatError(format!("头部长度 {} 不够", len)));
        }
        let header = bytes[2..(2 + len)].try_into().unwrap();
        let header: Header = serde_json::from_slice(header)?;
        Ok(Self {
            header,
            body: bytes[(2 + len)..].to_vec(),
        })
    }
}

#[test]
fn test_frame_build() {
    let header = Header {
        frame_type: FrameType::Response,
        frame_id: Some(1),
        session_id: Some("session_id".to_string()),
        service_type: Some("service_type".to_string()),
    };
    let body = vec![1, 2, 3];
    let frame = Frame::new(header, body.clone());
    let bytes = frame.to_vec().unwrap();
    println!("{}", String::from_utf8_lossy(&bytes));
    let frame = Frame::from_slice(&bytes).unwrap();
    println!("{:?}", frame);
    assert_eq!(frame.header.frame_type, FrameType::Response);
    assert_eq!(frame.header.frame_id, Some(1));
    assert_eq!(frame.header.session_id, Some("session_id".to_string()));
    assert_eq!(frame.header.service_type, Some("service_type".to_string()));
    assert_eq!(frame.body, body);
}
