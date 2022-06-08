use super::base::*;
use super::recv::*;
use crate::alink::{AlinkRequest, AlinkResponse, ParamsRequest, SysAck};
use crate::{Error, Result, ThreeTuple};
use regex::Regex;
use rumqttc::{AsyncClient, QoS};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

impl super::Module {
    pub async fn upload_init(&self, params: InitParams) -> crate::Result<InitData> {
        let payload: InitRequest = ParamsRequest::new(params);
        let topic = format!(
            "/sys/{}/{}/thing/file/upload/mqtt/init",
            self.three.product_key, self.three.device_name
        );
        let (tx, rx) = oneshot::channel();
        self.data.send((payload.id.clone(), tx)).await;
        self.publish(topic, &payload).await?;
        if let Ok(data) = rx.await {
            match data {
                FileRecv::InitReply(reply) => {
                    if reply.code == 200 {
                        return Ok(reply.data);
                    } else {
                        return Err(Error::CodeParams(reply.code, reply.message));
                    }
                }
                _ => {}
            }
        }
        Err(Error::WaitResponseTimeout(payload.id))
    }

    pub async fn upload_send(
        &self,
        params: SendHeaderParams,
        bytes: &[u8],
    ) -> crate::Result<SendReplyData> {
        if params.b_size != bytes.len() {
            return Err(Error::SizeNotMatch(params.b_size, bytes.len()));
        }
        let header: SendHeader = ParamsRequest::new(params);
        let id = header.id.clone();
        let payload = SendPayload::payload(header, bytes)?;
        let topic = format!(
            "/sys/{}/{}/thing/file/upload/mqtt/send",
            self.three.product_key, self.three.device_name
        );
        let (tx, rx) = oneshot::channel();
        self.data.send((id.clone(), tx)).await;
        self.publish_raw(topic, payload).await?;
        if let Ok(data) = rx.await {
            match data {
                FileRecv::SendReply(reply) => {
                    if reply.code == 200 {
                        return Ok(reply.data);
                    } else {
                        return Err(Error::CodeParams(reply.code, reply.message));
                    }
                }
                _ => {}
            }
        }
        Err(Error::WaitResponseTimeout(id))
    }

    pub async fn upload_cancel(&self, params: UploadId) -> crate::Result<UploadId> {
        let payload: CancelRequest = ParamsRequest::new(params);
        let topic = format!(
            "/sys/{}/{}/thing/file/upload/mqtt/cancel",
            self.three.product_key, self.three.device_name
        );
        let (tx, rx) = oneshot::channel();
        self.data.send((payload.id.clone(), tx)).await;
        self.publish(topic, &payload).await?;
        if let Ok(data) = rx.await {
            match data {
                FileRecv::CancelReply(reply) => {
                    if reply.code == 200 {
                        return Ok(reply.data);
                    } else {
                        return Err(Error::CodeParams(reply.code, reply.message));
                    }
                }
                _ => {}
            }
        }
        Err(Error::WaitResponseTimeout(payload.id))
    }
}

/// 设备请求上传文件
/// 请求Topic：`/sys/${productKey}/${deviceName}/thing/file/upload/mqtt/init`。
pub type InitRequest = ParamsRequest<InitParams>;

// 设备上传文件分片
// 请求Topic：`/sys/${productKey}/${deviceName}/thing/file/upload/mqtt/send`。
// SendPayload

/// 设备取消上传文件
/// 请求Topic：/sys/${productKey}/${deviceName}/thing/file/upload/mqtt/cancel。
pub type CancelRequest = ParamsRequest<UploadId>;
