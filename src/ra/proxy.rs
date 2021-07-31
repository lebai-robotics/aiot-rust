use super::protocol;
use super::protocol::{ErrorCode, LocalServiceInfo, MsgHead, MsgResponse, MsgType};
use super::session::{SessionId, SessionList};
use super::{Error, RemoteAccessOptions, Result};
use crate::util::{auth, rand_string};
use log::*;
use rustls::ClientConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

const DEFAULT_MSG_ID_HDSK: &str = "msg_id_for_handshake";
const CLOUD_CHANNEL_KEEPALIVE_CNT_MAX: u8 = 3;

pub struct RemoteAccessProxy {
    session_list: SessionList,
    local_tx: Sender<(String, Vec<u8>)>,
    local_rx: Receiver<(String, Vec<u8>)>,
    cloud_rx: Receiver<Vec<u8>>,
    params: RemoteAccessOptions,
    local_services: Vec<LocalServiceInfo>,
    client_config: Arc<ClientConfig>,
}

impl RemoteAccessProxy {
    pub fn new(cloud_rx: Receiver<Vec<u8>>, params: RemoteAccessOptions) -> Result<Self> {
        let (local_tx, local_rx) = mpsc::channel(128);
        Ok(Self {
            cloud_rx,
            local_tx,
            local_rx,
            params,
            client_config: Arc::new(auth::aliyun_client_config()?),
            local_services: vec![Default::default()],
            session_list: Default::default(),
        })
    }

    pub async fn process(&mut self) -> Result<()> {
        let data = self.cloud_rx.recv().await.unwrap();
        let data: EdgeDebugSwitch = serde_json::from_slice(&data)?;
        debug!("{:?}", data);
        if data.status != 0 {
            self.create_remote_proxy().await?;
        }
        Ok(())
    }

    /// 创建北向云端连接
    async fn create_remote_proxy(&mut self) -> Result<()> {
        use futures_util::{SinkExt, StreamExt};
        use tokio::time;
        use tokio::time::Duration;
        use tokio_tungstenite::client_async_tls_with_config;
        use tokio_tungstenite::Connector;

        // 创建云端通道连接资源
        let addr = format!("{}:{}", self.params.cloud_host, self.params.cloud_port);
        let payload = protocol::handshake_payload(
            &self.params.pk,
            &self.params.dn,
            &self.params.ds,
            &self.local_services,
        )?;
        let data = protocol::gen_response(
            MsgType::ServiceProviderConnReq,
            &DEFAULT_MSG_ID_HDSK,
            "",
            payload.as_bytes(),
        )?;

        let url = url::Url::parse(&format!("wss://{}/", addr))?;
        let socket = TcpStream::connect(addr).await?;
        let connecter = Connector::Rustls(self.client_config.clone());
        let (ws_stream, _) =
            client_async_tls_with_config(url, socket, None, Some(connecter)).await?;

        let (mut write, mut read) = ws_stream.split();
        write
            .send(data.into())
            .await
            .map_err(|_| Error::UploadDataError)?;

        let mut keepalive_cnt: u8 = 0;
        let mut interval = time::interval(Duration::from_secs(60));
        loop {
            tokio::select! {
                Some(data) = self.cloud_rx.recv() => {
                    let data: EdgeDebugSwitch = serde_json::from_slice(&data)?;
                    debug!("{:?}", data);
                    if data.status == 0 {
                        info!("关闭远程登录");
                        return Ok(());
                    }
                },
                Some((token, data)) = self.local_rx.recv() => {
                    // 本地服务的数据的处理
                    // debug!("local_rx[{}]<-{:x?}", token, data);
                    write.send(protocol::gen_response(MsgType::ServiceProviderRawProtocol, "", &token, &data)?.into()).await?;
                },
                Some(Ok(msg)) = read.next() => {
                    match protocol::parse_message(msg.into_data()) {
                        Ok((header, payload)) => {
                            debug!("header: {:?}", header);
                            // 云端下行数据和命令的处理
                            match header.msg_type {
                                MsgType::ServiceConsumerNewSession => {
                                    // 命令：本地服务session的开启
                                    let s = self.new_session(&header, &payload).await?;
                                    write.send(s.into()).await?;
                                },
                                MsgType::ServiceConsumerReleaseSession => {
                                    // 命令：则做本地服务session的关闭
                                    if let Some(token) = header.token {
                                        let c = token.clone();
                                        let id = self.session_list.release(token);
                                        let data = serde_json::to_string(&id)?;
                                        let payload = protocol::response_payload(ErrorCode::Ok, Some(&data), Some("release session response"))?;
                                        write.send(protocol::gen_response(MsgType::RespOk, &header.msg_id, &c, payload.as_bytes())?.into()).await?;
                                    }
                                },
                                MsgType::ServiceConsumerRawProtocol => {
                                    // 数据：云端传输给本地服务的数据
                                    match header.token {
                                        Some(token) => {
                                            if let Err(err) = self.session_list.write(token, payload).await {
                                                error!("write err: {}", err);
                                                write.send(protocol::gen_error(ErrorCode::SessionNonexistent, None, &header.msg_id)?.into()).await?;
                                            }
                                        },
                                        None => {
                                            write.send(protocol::gen_error(ErrorCode::ParamInvalid, None, &header.msg_id)?.into()).await?;
                                        }
                                    }
                                },
                                MsgType::RespOk => {
                                    // 命令：云通道上线后握手协议的response
                                    let body: MsgResponse = serde_json::from_slice(&payload)?;
                                    info!("body {:?}", body);
                                    if header.msg_id == DEFAULT_MSG_ID_HDSK {
                                        // 握手成功，设置每 20s 发送一次心跳报文
                                        interval = time::interval(Duration::from_secs(20));
                                    }
                                },
                                MsgType::ServiceVerifyAccount => {
                                },
                                MsgType::KeepalivePong => {
                                    keepalive_cnt = 0;
                                },
                                _ => {
                                    warn!("recv error ws package: {:?}", header);
                                }
                            }
                        },
                        Err(err) => error!("{}", err),
                    }
                },
                _ = interval.tick() => {
                    if keepalive_cnt > CLOUD_CHANNEL_KEEPALIVE_CNT_MAX {
                        return Err(Error::KeepaliveTimeout);
                    }
                    keepalive_cnt += 1;
                    // 发送通道保活信号
                    let n = "";
                    let msg_id = rand_string(32);
                    write.send(protocol::gen_response(MsgType::KeepalivePing, &msg_id, &n, n.as_bytes())?.into()).await?;
                }
            }
        }
    }

    async fn new_session(&mut self, head: &MsgHead, body: &[u8]) -> Result<Vec<u8>> {
        debug!("new session payload: {}", String::from_utf8_lossy(&body));
        let body = serde_json::from_slice(&body).unwrap_or_default();
        match self.session_list.new(&body, self.local_tx.clone()).await {
            Ok(id) => {
                let session_id = SessionId::new(&id);
                let rep = serde_json::to_string(&session_id)?;
                let payload = protocol::response_payload(ErrorCode::Ok, Some(&rep), None)?;
                protocol::gen_response(MsgType::RespOk, &head.msg_id, &id, payload.as_bytes())
            }
            Err(err) => protocol::gen_error(
                ErrorCode::SessionCreateFailed,
                Some(&err.to_string()),
                &head.msg_id,
            ),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct EdgeDebugSwitch {
    status: i32,
}
