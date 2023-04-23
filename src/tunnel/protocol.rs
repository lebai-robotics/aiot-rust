use crate::{util::rand_u64, Error, Result};
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

impl Service {
    pub fn new(r#type: String, ip: String, port: u16) -> Self {
        Self { r#type, ip, port }
    }
}

impl Default for Service {
    fn default() -> Self {
        Self::new("_SSH".into(), "127.0.0.1".into(), 22)
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

/// 响应结果码，取值范围0~255，0~15为系统预留响应码，16~255可由您自定义。
#[derive(Serialize_repr, Deserialize_repr, Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum ResponseCode {
    /// 0：表示创建Session成功，其他表示失败。
    Success = 0,
    /// 1：表示物联网平台的云端识别到单个安全隧道中Session数量已达到上限（10个），无法再创建。
    SessionLimit = 1,
    /// 2：表示设备端拒绝创建该Session。
    DeviceRefused = 2,
}

/// 关闭Session的原因
#[derive(Serialize_repr, Deserialize_repr, Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum ReleaseCode {
    /// 0：表示访问端主动关闭Session。
    ClientClose = 0,
    /// 1：表示设备端主动关闭Session。
    DeviceClose = 1,
    /// 2：表示物联网平台因检测到访问端断连，关闭Session。
    CloudClientDisconnect = 2,
    /// 3：表示物联网平台因检测到设备端断连，关闭Session。
    CloudDeviceDisconnect = 3,
    /// 4：表示物联网平台因系统更新，关闭Session，设备端和访问端可以延时1秒后重新建连。
    CloudUpdate = 4,
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
    pub fn session_id(&self) -> String {
        self.header.session_id.clone().unwrap_or("".to_string())
    }

    pub fn service_type(&self) -> String {
        self.header.service_type.clone().unwrap_or("".to_string())
    }

    pub fn frame_id(&self) -> u64 {
        self.header.frame_id.clone().unwrap_or(rand_u64())
    }

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

    pub fn raw(
        session_id: String,
        frame_id: u64,
        service_type: Option<String>,
        body: Vec<u8>,
    ) -> Frame {
        Frame::new(
            Header {
                frame_type: FrameType::RawData,
                frame_id: Some(frame_id),
                session_id: Some(session_id),
                service_type,
            },
            body,
        )
    }

    pub fn response(
        session_id: String,
        frame_id: u64,
        service_type: String,
        code: ResponseCode,
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

    pub fn release(session_id: String, frame_id: u64, code: ReleaseCode, msg: String) -> Frame {
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
        log::debug!("{}{}", header.len(), String::from_utf8_lossy(&header));
        Ok(buf)
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 2 {
            return Err(Error::HeaderFormatError("长度不够".into()));
        }
        let len = u16::from_be_bytes(bytes[..2].try_into().unwrap()) as usize;
        if bytes.len() < 2 + len {
            return Err(Error::HeaderFormatError(format!(
                "头部长度 {} + 2 不够 {} [{:02x}][{:02x}]",
                bytes.len(),
                len,
                bytes[0],
                bytes[1]
            )));
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
