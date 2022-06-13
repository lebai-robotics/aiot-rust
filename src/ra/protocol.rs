use super::{Error, Result};
use crate::util::auth;
use crate::util::timestamp;
use log::*;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

const G_UUID: &str = "alibaba_iot";
const VERSION: &str = "2.1";
const RNRN: &str = "\r\n\r\n";

/// 本地服务类型的抽象描述
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LocalServiceInfo {
    pub r#type: String,
    pub ip: String,
    pub port: u16,
}

impl Default for LocalServiceInfo {
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
pub struct MsgHead {
    /// 隧道帧类型。可取值为：
    /// 1. common_response，响应数据。
    /// 2. session_create，创建Session。
    /// 3. session_release，关闭Session。
    /// 4. data_transport，创建Session内的数据传输。
    pub frame_type: MsgType,
    /// 访问端或设备端发送通信数据时设置的帧ID，取值范围为0~（263-1）。
    /// 建议设备端和访问端均使用递增的帧ID，用于区分每个session_id会话中的通信数据。
    pub frame_id: u64,
    /// 不同类型隧道帧的会话ID，在当前安全隧道内唯一。
    /// 访问端发送创建Session的请求帧时，不需要传入该参数，物联网平台会根据收到的请求帧分配一个会话ID，并发送给设备端。其他类型的隧道帧，访问端和设备端均需要传递会话ID。
    pub session_id: Option<String>,
    /// Session对应的业务类型，由您自定义。支持英文字母、下划线（_）、短划线（-）和英文句号（.），首字母必须为英文字母，最长不超过16个字符。
    pub service_type: Option<String>,
}

/// MSG_RESP_OK响应报文格式
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MsgResponse {
    pub code: ErrorCode,
    pub msg: String,
}

/// MSG_SERVICE_PROVIDER_CONN_REQ握手报文格式
#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
pub struct MsgHandshake<'a> {
    pub uuid: &'a str,
    pub product_key: &'a str,
    pub device_name: &'a str,
    pub version: &'a str,
    pub IP: &'a str,
    pub MAC: &'a str,
    pub session_id: &'a str,
    pub service_meta: &'a [LocalServiceInfo],
    pub signmethod: &'a str,
    pub sign: &'a str,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Copy, Clone, PartialEq)]
#[repr(u16)]
pub enum MsgType {
    /// 消息的response
    RespOk = 0,
    /// common_response，响应数据。
    CommonResponse = 1,
    /// 新增session
    ServiceConsumerNewSession = 2,
    /// 释放session
    ServiceConsumerReleaseSession = 3,
    /// 服务发送的原始服务协议.
    ServiceProviderRawProtocol = 4,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Copy, Clone, PartialEq)]
#[repr(i32)]
pub enum ErrorCode {
    Ok = 0,
    //成功
    SignatureInvalid = 101600,
    //签名验证失败
    ParamInvalid = 101601,
    //入参不合法
    SessionLimit = 101602,
    //Session已达最大值
    SessionNonexistent = 101603,
    //Session不存在
    SessionCreateFailed = 101604,
    //Session创建失败
    // ServiceUnavalibe = 101604,        //服务不可达
    ServiceExit = 101605,
    //服务异常退出
    ConnectionClose = 101606,
    //连接异常退出
    VerifyAccout = 101607,
    //校验账号失败
    BackendServiceUnavalibe = 101671, //backend service not available
}

pub fn handshake_payload(
    pk: &str,
    dn: &str,
    ds: &str,
    service_meta: &[LocalServiceInfo],
) -> Result<String> {
    let sign = auth::sign_device(G_UUID, dn, pk, ds, timestamp() as u128);
    let msg = MsgHandshake {
        uuid: G_UUID,
        product_key: pk,
        device_name: dn,
        version: VERSION,
        IP: "",
        MAC: "",
        session_id: "",
        service_meta,
        signmethod: auth::SIGN_METHOD,
        sign: &sign,
    };
    serde_json::to_string(&msg).map_err(Error::SerdeError)
}

pub fn response_payload(code: ErrorCode, data: Option<&str>, msg: Option<&str>) -> Result<String> {
    let res = MsgResponse {
        code,
        msg: msg.unwrap_or("null").to_string(),
    };
    let res = serde_json::to_string(&res)?;
    debug!("res: {}", res);
    Ok(format!("{}{}{}", res, RNRN, data.unwrap_or("null")))
}

pub fn parse_message(msg: Vec<u8>) -> Result<(MsgHead, Vec<u8>)> {
    // debug!("msg: {:x?}", msg);
    let hdr_len = ((msg[0] as usize) << 8) | (msg[1] as usize);
    let mut iter = msg[2..].iter();
    // let start = iter.position(|&x| x == b'{').ok_or(anyhow!("少左括号"))?;
    let end = iter
        .position(|&x| x == b'}')
        .ok_or(Error::HeaderFormatError("少右括号".into()))?;
    let (h, p) = msg.split_at(end + 1);
    // debug!(
    //     "h={}, p({})={}",
    //     String::from_utf8_lossy(&h),
    //     p.len(),
    //     String::from_utf8_lossy(&p)
    // );
    let header: MsgHead = serde_json::from_slice(h)?;
    Ok((header, p.to_vec()))
}

pub fn gen_response(
    frame_type: MsgType,
    frame_id: u64,
    session_id: Option<String>,
    service_type: Option<String>,
    payload: &[u8],
) -> Result<Vec<u8>> {
    let msg = MsgHead {
        frame_type,
        frame_id,
        session_id,
        service_type,
    };
    let header = serde_json::to_vec(&msg).map_err(Error::SerdeError)?;
    let header_length = header.len();

    let mut data = Vec::new();
    data.extend_from_slice(&header_length.to_be_bytes());
    data.extend_from_slice(&header);
    // let s = format!("{}{}{}", data, RNRN, String::f payload);
    data.extend_from_slice(&payload);
    // debug!("响应给云端={}", String::from_utf8_lossy(&s));
    Ok(data)
}

pub fn gen_error(code: ErrorCode, msg: Option<&str>, frame_id: u64) -> Result<Vec<u8>> {
    let payload = response_payload(code, None, msg)?;
    gen_response(MsgType::RespOk, frame_id, None, None, payload.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 测试serde_json() {
        let b = r#"{"frame_id":"MSG_FRONTEND_CONN_REQ_0.9149057932059996","frame_type":4,"payload_len":114,"service_type":1,"timestamp":0}"#;
        let x = serde_json::from_str::<MsgHead>(b).unwrap();
        println!("{:?}", x);
    }
}
