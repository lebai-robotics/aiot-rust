//! 文件上传

use crate::alink::aiot_module::{AiotModule, ModuleRecvKind};
use crate::alink::{AlinkRequest, AlinkResponse};
use crate::mqtt::MqttConnection;
use crate::{Error, Result, ThreeTuple};
use enum_iterator::IntoEnumIterator;
use rumqttc::{AsyncClient, QoS};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{broadcast, mpsc, oneshot};

use self::recv::*;

pub mod base;
pub mod push;
pub mod recv;
pub mod util;

pub use base::*;

pub type Recv = FileRecv;
pub type RecvKind = FileRecvKind;
pub type Module = AiotModule<Recv, Sender<(String, oneshot::Sender<Recv>)>>;

pub const CHUNK_SIZE: usize = 4096;

impl Module {
    pub async fn init(&self) -> Result<()> {
        self.sub_all::<RecvKind>().await
    }

    pub async fn upload(&self, path: impl AsRef<std::path::Path>) -> Result<String> {
        use tokio::fs::File;
        use tokio::io::{AsyncReadExt, AsyncSeekExt};

        let params = InitParams {
            file_name: util::filename_for_path(&path)?,
            file_size: -1,
            conflict_strategy: Some(ConflictStrategy::Overwrite),
            ..Default::default()
        };
        let info = self.upload_init(params).await?;
        log::info!("upload_init: {:?}", info);

        let mut file = File::open(path).await?;

        let mut buf = [0; CHUNK_SIZE];
        let mut offset = 0;
        let mut is_complete = Some(false);
        if let Some(seek) = info.offset {
            file.seek(std::io::SeekFrom::Start(seek as u64)).await?;
            offset = seek as usize;
        }
        loop {
            let b_size = file.read(&mut buf).await?;
            if b_size < CHUNK_SIZE {
                is_complete = Some(true);
            }
            // log::debug!("read: {} [{}]", b_size, String::from_utf8_lossy(&buf));
            let params = SendHeaderParams {
                upload_id: info.upload_id.clone(),
                offset,
                b_size,
                is_complete,
            };
            offset += b_size;
            if let Err(err) = self.upload_send(params.clone(), buf[..b_size].into()).await {
                log::error!("upload_send: {:?}", err);
                self.upload_send(params, buf[..b_size].into()).await?;
            }
            if is_complete.unwrap_or(false) {
                break;
            }
        }
        Ok(info.file_name)
    }
}

impl MqttConnection {
    pub fn file_uploader(&mut self) -> Result<Module> {
        let (tx, rx) = mpsc::channel(64);
        let (tx_, rx_) = mpsc::channel(64);
        let executor = Executor {
            tx,
            rx_,
            three: self.mqtt_client.three.clone(),
            map: HashMap::new(),
        };
        self.module(Box::new(executor), rx, tx_)
    }
}

pub struct Executor {
    three: Arc<ThreeTuple>,
    tx: Sender<Recv>,
    rx_: Receiver<(String, oneshot::Sender<Recv>)>,
    map: HashMap<String, oneshot::Sender<Recv>>,
}

#[async_trait::async_trait]
impl crate::Executor for Executor {
    async fn execute(&mut self, topic: &str, payload: &[u8]) -> crate::Result<()> {
        while let Ok(r) = self.rx_.try_recv() {
            self.map.insert(r.0, r.1);
        }

        let data = crate::execute::<RecvKind>(&self.three, topic, payload)?;

        let id = match &data {
            Recv::InitReply(item) => item.id.clone(),
            Recv::SendReply(item) => item.id.clone(),
            Recv::CancelReply(item) => item.id.clone(),
        };
        if let Some(id) = id {
            if let Some(item) = self.map.remove(&id) {
                item.send(data.clone());
            }
        } else {
            if let Some((id, _)) = self.map.iter().next() {
                let id = id.clone();
                if let Some(item) = self.map.remove(&id) {
                    item.send(data.clone());
                }
            }
        }

        self.tx.send(data).await.map_err(|_| Error::MpscSendError)
    }
}
