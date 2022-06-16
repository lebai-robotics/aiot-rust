use super::protocol::Service;
use super::session::{Session, SessionList};
use crate::tunnel::protocol::{Frame, FrameType, ReleaseCode};
use crate::util::auth::aliyun_client_config;
use crate::util::rand_u64;
use crate::Result;
use futures::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::RwLock;
use tokio::time;
use tokio::time::Duration;
use tokio_tungstenite::Connector;
use tokio_tungstenite::{client_async_tls_with_config, MaybeTlsStream, WebSocketStream};
use tungstenite::Message;

pub struct TunnelParams {
    id: String,
    host: String,
    port: String,
    path: String,
    token: String,
}

pub enum TunnelAction {
    AddTunnel(TunnelParams),
    UpdateTunnel(TunnelParams),
    DeleteTunnel(String),
    AddService(Service),
    UpdateService(Service),
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

    pub fn add_tunnel(&self) -> Result<String> {
        Ok("".into())
    }

    pub fn delete_tunnel(&self, id: &str) -> Result<()> {
        Ok(())
    }

    pub fn update_tunnel(&self, id: &str) -> Result<()> {
        Ok(())
    }
}

pub struct RemoteAccessProxy {
    params: TunnelParams,
    read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    action_rx: Receiver<ProxyAction>,
    cloud_rx: Receiver<Vec<u8>>,
    local_tx: Sender<(String, String, Vec<u8>)>,
    local_rx: Receiver<(String, String, Vec<u8>)>,
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
                    TunnelAction::UpdateService(service) => {
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
                    TunnelAction::AddTunnel(params) => {
                        let (tx, rx) = mpsc::channel(16);
                        proxytxs.insert(params.id.clone(), tx);
                        if let Err(err) = Self::new(params, rx, client_config.clone()).await {
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
        client_config: Arc<rustls::ClientConfig>,
    ) -> Result<()> {
        let addr = params.path.as_str();
        let url = url::Url::parse(&format!("wss://{}/", addr))?;
        let socket = TcpStream::connect(addr).await?;
        let connecter = Connector::Rustls(client_config);
        let (ws_stream, _) =
            client_async_tls_with_config(url, socket, None, Some(connecter)).await?;
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

    pub async fn poll(&mut self) -> Result<()> {
        tokio::select! {
            Some(Ok(data)) = self.read.next() => {
                let data = Frame::from_slice(&data.into_data())?;
                match data.header.frame_type {
                    FrameType::Response => {},
                    FrameType::NewSession => if let Some(id) = data.header.session_id {
                        if let Some(service) = SERVICE_LIST.read().await.get(&self.params.id) {
                            self.session_list.add(id, service, self.local_tx.clone()).await.ok();
                        }
                    },
                    FrameType::ReleaseSession => if let Some(id) = data.header.session_id {
                        self.session_list.release(id).await.ok();
                    },
                    FrameType::RawData => if let Some(id) = data.header.session_id {
                        self.session_list.write(id, data.body.clone()).await.ok();
                    },
                }
                Ok(())
            },
            Some((id, service_type, data)) = self.local_rx.recv() => {
                self.write.send(Frame::raw(id, rand_u64(), service_type, data).to_vec()?.into()).await.ok();
                Ok(())
            },
            Some(data) = self.action_rx.recv() => {
                match data {
                    ProxyAction::UpdateTunnel(params) => {
                        self.params = params;
                    }
                    ProxyAction::DeleteTunnel(id) => {
                        self.exit_flag = true;
                        self.write.send(Frame::release(id, rand_u64(), ReleaseCode::DeviceClose, "".into()).to_vec()?.into()).await.ok();
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
