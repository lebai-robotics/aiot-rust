use super::protocol::{Frame, ReleaseCode, Service};
use crate::util::inc_u64;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Sender};
use tokio::task;

pub struct Session {
    tx: Sender<Vec<u8>>,
    service_type: String,
    local_tx: Sender<Frame>,
}

pub struct SessionList {
    txs: HashMap<String, Session>,
}

impl SessionList {
    pub fn new() -> Self {
        Self {
            txs: HashMap::new(),
        }
    }
}

impl SessionList {
    pub async fn add(&mut self, id: String, info: &Service, local_tx: Sender<Frame>) -> Result<()> {
        let addr = format!("{}:{}", info.ip, info.port);
        log::info!("tcp://{} session_id: {}", addr, id);

        let (tx, mut rx) = mpsc::channel(128);
        let service_type = info.r#type.clone();
        self.txs.insert(
            id.clone(),
            Session {
                tx,
                service_type: service_type.clone(),
                local_tx: local_tx.clone(),
            },
        );

        let mut stream = TcpStream::connect(&addr).await?;
        task::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                tokio::select! {
                    Some(w) = rx.recv() => {
                        // log::debug!("write {}={}", addr, String::from_utf8_lossy(&w));
                        if let Err(err) = stream.write_all(&w).await {
                            log::error!("write error: {:?}", err);
                        }
                    },
                    Ok(n) = stream.read(&mut buf) => {
                        // log::debug!("read={:x?}", &buf[..n]);
                        if n == 0 {
                            let frame = Frame::release(id.clone(), inc_u64(), ReleaseCode::DeviceClose, "server closed".to_string());
                            local_tx.send(frame).await.ok();
                            return;
                        }
                        let frame = Frame::raw(id.clone(), inc_u64(), Some(service_type.clone()), buf[..n].into());
                        local_tx.send(frame).await.ok();
                    },
                    else => break,
                }
            }
            let frame = Frame::release(id.clone(), inc_u64(), ReleaseCode::DeviceClose, "unknown".to_string());
            local_tx.send(frame).await.ok();
        });

        Ok(())
    }

    pub async fn release(&mut self, id: String, code: ReleaseCode, msg: String) -> Result<()> {
        self.txs.remove(&id);
        let data = Frame::release(id.clone(), inc_u64(), code, msg);

        let session = self
            .txs
            .get(&id)
            .ok_or(Error::SessionNotFound(id.to_string()))?
            .clone();
        session
            .local_tx
            .send(data)
            .await
            .map_err(|_| Error::MpscSendError)
    }

    pub async fn write(&mut self, id: String, data: Vec<u8>) -> Result<()> {
        let session = self
            .txs
            .get(&id)
            .ok_or(Error::SessionNotFound(id.to_string()))?
            .clone();
        session
            .tx
            .send(data)
            .await
            .map_err(|_| Error::UploadDataError)?;
        Ok(())
    }
}
