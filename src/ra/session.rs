use super::protocol::LocalServiceInfo;
use super::{Error, Result};
use crate::util;
use log::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::task;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SessionId {
    pub session_id: String,
}

impl SessionId {
    pub fn new(session_id: &str) -> Self {
        Self {
            session_id: session_id.to_string(),
        }
    }
}

pub struct SessionList {
    txs: HashMap<String, Sender<Vec<u8>>>,
}

impl Default for SessionList {
    fn default() -> Self {
        Self {
            txs: HashMap::new(),
        }
    }
}

impl SessionList {
    pub async fn new(
        &mut self,
        info: &LocalServiceInfo,
        local_tx: Sender<(String, Vec<u8>)>,
    ) -> Result<String> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let addr = format!("{}:{}", info.ip, info.port);
        let id = util::rand_string(32);
        let res = id.clone();
        debug!("{} session_id: {:?}", addr, id);

        let (tx, mut rx) = mpsc::channel(128);
        self.txs.insert(id.clone(), tx);

        let mut stream = TcpStream::connect(&addr).await?;
        task::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                tokio::select! {
                    Some(w) = rx.recv() => {
                        stream.write_all(&w).await.map_err(|_| Error::MpscSendError)?;
                        debug!("write {}={:x?}", addr, w);
                    },
                    Ok(n) = stream.read(&mut buf) => {
                        debug!("read={:x?}", &buf[0..n]);
                        if n == 0 {
                            break;
                        }
                        local_tx.send((id.clone(), (&buf[0..n]).to_vec())).await.map_err(|_| Error::MpscSendError)?;
                    },
                    else => break,
                }
            }
            info!("Session {} 已退出", id);
            Ok::<_, Error>(())
        });

        Ok(res)
    }

    pub fn release(&mut self, id: String) -> SessionId {
        self.txs.remove(&id);
        SessionId { session_id: id }
    }

    pub async fn write(&mut self, id: String, data: Vec<u8>) -> Result<()> {
        let tx = self.txs.get(&id).ok_or(Error::SessionNotFound(id))?.clone();
        tx.send(data).await.map_err(|_| Error::UploadDataError)?;
        Ok(())
    }
}
