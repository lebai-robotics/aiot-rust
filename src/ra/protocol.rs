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
    pub service_type: String,
    pub service_name: String,
    pub service_ip: String,
    pub service_port: u16,
}

impl Default for LocalServiceInfo {
    fn default() -> Self {
        Self {
            service_type: "SSH".to_string(),
            service_name: "ssh_localhost".to_string(),
            service_ip: "127.0.0.1".to_string(),
            service_port: 22,
        }
    }
}

/// 消息报文头格式-->通用
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MsgHead {
    pub msg_type: MsgType,
    pub payload_len: usize,
    pub msg_id: String,
    pub timestamp: u64,
    pub token: Option<String>,
}

/// MSG_RESP_OK响应报文格式
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MsgResponse {
    pub code: ErrorCode,
    pub message: String,
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
    pub token: &'a str,
    pub service_meta: &'a [LocalServiceInfo],
    pub signmethod: &'a str,
    pub sign: &'a str,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Copy, Clone, PartialEq)]
#[repr(u16)]
pub enum MsgType {
    RespOk = 0,
    //消息的response
    ServiceProviderConnReq = 1,
    //服务提供者发起（握手）连接请求.
    ServiceConsumerConnReq = 2,
    //服务消费者发起（握手）连接请求.
    ServiceConsumerNewSession = 4,
    //新增session
    ServiceConsumerReleaseSession = 5,
    //释放session
    ServiceVerifyAccount = 6,
    //云端向边缘设备发起验证账号、密码的请求
    ServiceProviderRawProtocol = 21,
    //服务提供者发送的原始服务协议.
    ServiceConsumerRawProtocol = 22,
    //服务消费者发送的原始服务协议.
    KeepalivePing = 256,
    //心跳PING
    KeepalivePong = 257, //心跳PONG
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
        token: "",
        service_meta,
        signmethod: auth::SIGN_METHOD,
        sign: &sign,
    };
    serde_json::to_string(&msg).map_err(Error::SerdeError)
}

pub fn response_payload(code: ErrorCode, data: Option<&str>, msg: Option<&str>) -> Result<String> {
    let res = MsgResponse {
        code,
        message: msg.unwrap_or("null").to_string(),
    };
    let res = serde_json::to_string(&res)?;
    debug!("res: {}", res);
    Ok(format!("{}{}{}", res, RNRN, data.unwrap_or("null")))
}

pub fn header(msg_type: MsgType, payload_len: usize, msg_id: &str, token: &str) -> Result<String> {
    let msg = MsgHead {
        msg_type,
        payload_len,
        msg_id: msg_id.to_string(),
        timestamp: timestamp(),
        token: Some(token.to_string()),
    };
    serde_json::to_string(&msg).map_err(Error::SerdeError)
}

pub fn parse_message(msg: Vec<u8>) -> Result<(MsgHead, Vec<u8>)> {
    // debug!("msg: {:x?}", msg);
    let mut iter = msg.iter();
    // let start = iter.position(|&x| x == b'{').ok_or(anyhow!("少左括号"))?;
    let end = iter
        .position(|&x| x == b'}')
        .ok_or(Error::HeaderFormatError("少右括号".into()))?;
    let (h, p) = msg.split_at(end + 1);
    let (_, p) = p.split_at(RNRN.len());
    // debug!(
    //     "h={}, p({})={}",
    //     String::from_utf8_lossy(&h),
    //     p.len(),
    //     String::from_utf8_lossy(&p)
    // );
    let header: MsgHead = serde_json::from_slice(h)?;
    if header.payload_len != p.len() {
        return Err(Error::HeaderFormatError(format!(
            "声明长度 {} != 实际长度 {}",
            header.payload_len,
            p.len()
        )));
    }
    Ok((header, p.to_vec()))
}

pub fn gen_response(
    msg_type: MsgType,
    msg_id: &str,
    token: &str,
    payload: &[u8],
) -> Result<Vec<u8>> {
    let header = header(msg_type, payload.len(), msg_id, token)?;
    // let s = format!("{}{}{}", header, RNRN, String::f payload);
    let s = [header.as_bytes(), payload].join(RNRN.as_bytes());
    // debug!("响应给云端={}", String::from_utf8_lossy(&s));
    Ok(s)
}

pub fn gen_error(code: ErrorCode, msg: Option<&str>, msg_id: &str) -> Result<Vec<u8>> {
    let payload = response_payload(code, None, msg)?;
    gen_response(MsgType::RespOk, msg_id, "", payload.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 测试serde_json() {
        let b = r#"{"msg_id":"MSG_FRONTEND_CONN_REQ_0.9149057932059996","msg_type":4,"payload_len":114,"service_type":1,"timestamp":0}"#;
        let x = serde_json::from_str::<MsgHead>(b).unwrap();
        println!("{:?}", x);
    }
}
