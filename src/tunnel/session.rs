use super::protocol::{Frame, ReleaseCode, Service};
use crate::util::rand_u64;
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
    pub async fn add(
        &mut self,
        id: String,
        info: &Service,
        local_tx: Sender<(String, String, Vec<u8>)>,
    ) -> Result<()> {
        let addr = format!("{}:{}", info.ip, info.port);
        log::info!("tcp://{} session_id: {}", addr, id);

        let (tx, mut rx) = mpsc::channel(128);
        let service_type = info.r#type.clone();
        self.txs.insert(
            id.clone(),
            Session {
                tx,
                service_type: service_type.clone(),
            },
        );

        let mut stream = TcpStream::connect(&addr).await?;
        task::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                tokio::select! {
                    Some(w) = rx.recv() => {
                        log::debug!("write {}={:x?}", addr, w);
                        if let Err(err) = stream.write_all(&w).await {
                            log::error!("write error: {:?}", err);
                        }
                    },
                    Ok(n) = stream.read(&mut buf) => {
                        log::debug!("read={:x?}", &buf[..n]);
                        if n == 0 {
                            break;
                        }
                        if let Err(err) = local_tx.send((id.clone(), service_type.clone(), (&buf[..n]).to_vec())).await {
                            log::error!("send to local: {}", err);
                        }
                    },
                    else => break,
                }
            }
            log::info!("Session {} 已退出", id);
            Ok::<_, Error>(())
        });

        Ok(())
    }

    pub async fn release(&mut self, id: String) -> Result<()> {
        self.txs.remove(&id);
        Ok(())
    }

    pub async fn write(&mut self, id: String, body: Vec<u8>) -> Result<()> {
        let session = self
            .txs
            .get(&id)
            .ok_or(Error::SessionNotFound(id.clone()))?
            .clone();
        session
            .tx
            .send(Frame::raw(id, rand_u64(), session.service_type.clone(), body).to_vec()?)
            .await
            .map_err(|_| Error::UploadDataError)?;
        Ok(())
    }
}
