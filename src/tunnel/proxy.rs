use super::protocol::{Header, Service};
use super::session::{Session, SessionList};
use crate::tunnel::protocol::{Frame, FrameType, ReleaseCode, ResponseCode, ResponseBody};
use crate::util::auth::aliyun_client_config;
use crate::util::inc_u64;
use crate::{Error, Result};
use futures::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::oneshot;
use tokio::sync::RwLock;
use tokio::time;
use tokio::time::Duration;
use tokio_tungstenite::Connector;
use tokio_tungstenite::{client_async_tls_with_config, MaybeTlsStream, WebSocketStream};
use tungstenite::Message;

pub struct TunnelParams {
    pub id: String,
    pub host: String,
    pub port: String,
    pub path: String,
    pub token: String,
}

pub enum TunnelAction {
    AddTunnel(TunnelParams, oneshot::Sender<String>),
    UpdateTunnel(TunnelParams),
    DeleteTunnel(String),
    AddService(Service),
    DeleteService(String),
}

enum ProxyAction {
    UpdateTunnel(TunnelParams),
    DeleteTunnel(String),
}

#[derive(Debug, Clone)]
pub struct TunnelProxy {
    tx: Sender<TunnelAction>,
}

impl TunnelProxy {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(16);
        tokio::spawn(async move {
            RemoteAccessProxy::start(rx).await;
        });
        Self { tx }
    }

    pub async fn add_tunnel(&self, params: TunnelParams) -> Result<String> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(TunnelAction::AddTunnel(params, tx))
            .await
            .map_err(|err| Error::MpscSendError)?;
        rx.await.map_err(|err| Error::OneshotRecvError)
    }

    pub async fn delete_tunnel(&self, id: &str) -> Result<()> {
        self.tx
            .send(TunnelAction::DeleteTunnel(id.to_string()))
            .await
            .map_err(|err| Error::MpscSendError)?;
        Ok(())
    }

    pub async fn update_tunnel(&self, params: TunnelParams) -> Result<()> {
        self.tx
            .send(TunnelAction::UpdateTunnel(params))
            .await
            .map_err(|err| Error::MpscSendError)?;
        Ok(())
    }

    pub async fn add_service(&self, service: Service) -> Result<()> {
        self.tx
            .send(TunnelAction::AddService(service))
            .await
            .map_err(|err| Error::MpscSendError)?;
        Ok(())
    }

    pub async fn delete_service(&self, id: &str) -> Result<()> {
        self.tx
            .send(TunnelAction::DeleteService(id.to_string()))
            .await
            .map_err(|err| Error::MpscSendError)?;
        Ok(())
    }
}

struct RemoteAccessProxy {
    params: TunnelParams,
    read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    action_rx: Receiver<ProxyAction>,
    cloud_rx: Receiver<Vec<u8>>,
    local_tx: Sender<Frame>,
    local_rx: Receiver<Frame>,
    one_tx: Option<oneshot::Sender<String>>, // 上送 sessionId
    exit_flag: bool,
    session_list: SessionList,
}

lazy_static! {
    static ref SERVICE_LIST: Arc<RwLock<HashMap<String, Service>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

impl RemoteAccessProxy {
    pub async fn start(mut rx: Receiver<TunnelAction>) {
        let client_config = Arc::new(aliyun_client_config().unwrap());
        let mut proxytxs: HashMap<String, Sender<ProxyAction>> = HashMap::new();
        loop {
            if let Some(action) = rx.recv().await {
                match action {
                    TunnelAction::AddService(service) => {
                        SERVICE_LIST
                            .write()
                            .await
                            .insert(service.r#type.clone(), service);
                    }
                    TunnelAction::DeleteService(id) => {
                        SERVICE_LIST.write().await.remove(&id);
                    }
                    TunnelAction::UpdateTunnel(params) => {
                        if let Some(tx) = proxytxs.get(&params.id) {
                            tx.send(ProxyAction::UpdateTunnel(params)).await.ok();
                        }
                    }
                    TunnelAction::DeleteTunnel(id) => {
                        if let Some(tx) = proxytxs.remove(&id) {
                            tx.send(ProxyAction::DeleteTunnel(id)).await.ok();
                        }
                    }
                    TunnelAction::AddTunnel(params, one_tx) => {
                        let (tx, rx) = mpsc::channel(16);
                        proxytxs.insert(params.id.clone(), tx);
                        if let Err(err) = Self::new(params, rx, one_tx, client_config.clone()).await
                        {
                            log::error!("add tunnel: {}", err);
                        }
                    }
                }
            }
        }
    }

    async fn new(
        params: TunnelParams,
        action_rx: Receiver<ProxyAction>,
        one_tx: oneshot::Sender<String>,
        client_config: Arc<rustls::ClientConfig>,
    ) -> Result<()> {
        let uri = format!("wss://{}:{}{}", params.host, params.port, params.path);
        let url = url::Url::parse(&uri)?;
        let addrs = url.socket_addrs(|| None)?;
        let socket = TcpStream::connect(&*addrs).await?;
        let connecter = Connector::Rustls(client_config);
        let mut request = http::request::Request::builder()
            .uri(uri)
            .header("tunnel-access-token", &params.token)
            .header("Sec-WebSocket-Protocol", "aliyun.iot.securetunnel-v1.1")
            .header(
                "Sec-WebSocket-Key",
                tungstenite::handshake::client::generate_key(),
            )
            .header("Host", &params.host)
            .header("Sec-WebSocket-Version", "13")
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .body(())
            .map_err(|err| Error::HttpRequestBuild)?;

        let (ws_stream, _) =
            client_async_tls_with_config(request, socket, None, Some(connecter)).await?;
        let (mut write, mut read) = ws_stream.split();

        let (cloud_tx, cloud_rx) = mpsc::channel(16);
        let (local_tx, local_rx) = mpsc::channel(16);
        let mut proxy = RemoteAccessProxy {
            params,
            read,
            write,
            action_rx,
            exit_flag: false,
            cloud_rx,
            local_tx,
            local_rx,
            one_tx: Some(one_tx),
            session_list: SessionList::new(),
        };
        tokio::spawn(async move {
            loop {
                if let Err(err) = proxy.poll().await {
                    log::warn!("proxy {} error: {err}", proxy.params.id);
                }
                if proxy.exit_flag {
                    log::info!("proxy {} exit", proxy.params.id);
                    break;
                }
            }
        });
        Ok(())
    }

    async fn new_session(&mut self, header: Header) -> Result<()> {
        let id = header.session_id.unwrap_or("".to_string());
        if let Some(tx) = self.one_tx.take() {
            tx.send(id.clone()).ok();
        }
        let service_type = header.service_type.unwrap_or("".to_string());
        if let Some(service) = SERVICE_LIST.read().await.get(&service_type) {
            if let Err(err) = self
                .session_list
                .add(id, service, self.local_tx.clone())
                .await
            {
                return Err(err);
            }
            return Ok(());
        }
        Err(Error::SessionCreate(format!(
            "找不到 service: {service_type}"
        )))
    }

    pub async fn poll(&mut self) -> Result<()> {
        tokio::select! {
            Some(Ok(data)) = self.read.next() => {
                let data = Frame::from_slice(&data.into_data())?;
                // log::info!("云端下发 {:?}", data);
                match data.header.frame_type {
                    FrameType::Response => {},
                    FrameType::NewSession => {
                        let data = match self.new_session(data.header.clone()).await {
                            Ok(()) => {
                                Frame::response(data.session_id(), data.frame_id(), data.service_type(), ResponseCode::Success, "new session response".to_string())
                            },
                            Err(err) => {
                                Frame::response(data.session_id(), data.frame_id(), data.service_type(), ResponseCode::DeviceRefused, format!("{err}"))
                            }
                        };
                        self.write.send(data.to_vec()?.into()).await.ok();
                    },
                    FrameType::ReleaseSession => if let Some(id) = data.header.session_id {
                        let code = if let Ok(body) = serde_json::from_slice::<ResponseBody<ReleaseCode>>(&data.body) {
                            body.code
                        } else {
                            ReleaseCode::DeviceClose
                        };
                        self.session_list.release(id, code, "".to_string()).await.ok();
                    },
                    FrameType::RawData => if let Some(id) = data.header.session_id {
                        self.session_list.write(id, data.body).await.ok();
                    },
                }
                Ok(())
            },
            Some(frame) = self.local_rx.recv() => {
                if let Err(err) = self.write.send(frame.to_vec()?.into()).await {
                    log::error!("send local data error: {}", err);
                }
                Ok(())
            },
            Some(data) = self.action_rx.recv() => {
                match data {
                    ProxyAction::UpdateTunnel(params) => {
                        self.params = params;
                    }
                    ProxyAction::DeleteTunnel(id) => {
                        self.exit_flag = true;
                        // let data = Frame::release(id, inc_u64(), ReleaseCode::DeviceClose, "".into());
                        // self.write.send(data.to_vec()?.into()).await.ok();
                    }
                }
                Ok(())
            },
            Some(data) = self.cloud_rx.recv() => {
                self.write.send(data.into()).await?;
                Ok(())
            }
            else => {
                Ok(())
            }
        }
    }
}
