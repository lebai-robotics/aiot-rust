use super::protocol::Service;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Sender};
use tokio::task;

pub struct SessionList {
    txs: HashMap<String, Sender<Vec<u8>>>,
    cloud_tx: Sender<Vec<u8>>,
}

impl SessionList {
    pub fn new(cloud_tx: Sender<Vec<u8>>) -> Self {
        Self {
            txs: HashMap::new(),
            cloud_tx
        }
    }
}

impl SessionList {
    pub async fn add(
        &mut self,
        id: String,
        info: &Service,
        local_tx: Sender<(String, Vec<u8>)>,
    ) -> Result<()> {
        let addr = format!("{}:{}", info.ip, info.port);
        log::debug!("tcp://{} session_id: {:?}", addr, id);

        let (tx, mut rx) = mpsc::channel(128);
        self.txs.insert(id.clone(), tx);

        let mut stream = TcpStream::connect(&addr).await?;
        task::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                tokio::select! {
                    Some(w) = rx.recv() => {
                        stream.write_all(&w).await.map_err(|_| Error::MpscSendError)?;
                        log::debug!("write {}={:x?}", addr, w);
                    },
                    Ok(n) = stream.read(&mut buf) => {
                        log::debug!("read={:x?}", &buf[0..n]);
                        if n == 0 {
                            break;
                        }
                        local_tx.send((id.clone(), (&buf[0..n]).to_vec())).await.map_err(|_| Error::MpscSendError)?;
                    },
                    else => break,
                }
            }
            log::info!("Session {} 已退出", id);
            Ok::<_, Error>(())
        });

        Ok(())
    }

    pub async fn release(&mut self, id: String) {
        self.txs.remove(&id);
        // self.cloud_tx.send(); 关闭Session
    }

    pub async fn write(&mut self, id: String, data: Vec<u8>) -> Result<()> {
        let tx = self.txs.get(&id).ok_or(Error::SessionNotFound(id))?.clone();
        tx.send(data).await.map_err(|_| Error::UploadDataError)?;
        Ok(())
    }
}
