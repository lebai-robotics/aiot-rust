//! 文件上传

use crate::alink::aiot_module::{AiotModule, ModuleRecvKind};
use crate::alink::{AlinkRequest, AlinkResponse};
use crate::mqtt::MqttConnection;
use crate::{Error, Result, ThreeTuple};
use enum_iterator::IntoEnumIterator;
use lazy_static::lazy_static;
use regex::Regex;
use rumqttc::{AsyncClient, QoS};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{broadcast, mpsc};

use self::recv::*;

pub mod base;
pub mod push;
pub mod recv;

pub use base::*;

pub type Recv = FileRecv;
pub type RecvKind = FileRecvKind;
pub type Module = AiotModule<Recv, broadcast::Sender<FileRecv>>;

lazy_static! {
    static ref REPLACE: Regex = Regex::new(r"[^a-zA-Z0-9_\.]").unwrap();
}

pub fn filename_for_path(path: impl AsRef<std::path::Path>) -> Result<String> {
    let path = path.as_ref();
    let filename = path.file_name().ok_or(Error::InvalidPath)?;
    let filename = filename.to_string_lossy().to_string();
    let filename = REPLACE.replace_all(&filename, regex::NoExpand("_"));
    let filename = filename.trim_start_matches("_");
    if filename.len() <= 0 {
        return Err(Error::InvalidPath);
    }
    let filename = if filename.len() > 100 {
        filename[..100].to_string()
    } else {
        filename.to_string()
    };
    Ok(filename)
}

#[test]
fn test1() {
    assert_eq!(
        filename_for_path(std::path::Path::new("/tmp/test.txt")).unwrap(),
        "test.txt"
    );
}
#[test]
fn test2() {
    assert_eq!(
        filename_for_path(std::path::Path::new("/tmp/_bc&^NU,HH.tar.gz")).unwrap(),
        "bc__NU_HH.tar.gz"
    );
}
#[test]
fn test3() {
    assert_eq!(
        filename_for_path(std::path::Path::new("/tmp/_你好.txt")).unwrap(),
        ".txt"
    );
}

#[test]
fn test4() {
    assert_eq!(
        filename_for_path(std::path::Path::new("/tmp/.env.dev")).unwrap(),
        ".env.dev"
    );
}

impl Module {
    pub async fn init(&self) -> Result<()> {
        // 这里特殊处理，因为阿里云对文件上传，如果传 /sys/+/+ 则会订阅失败
        let topics = [
            format!(
                "/sys/{}/{}/thing/file/upload/mqtt/init_reply",
                self.three.product_key, self.three.device_name
            ),
            format!(
                "/sys/{}/{}/thing/file/upload/mqtt/send_reply",
                self.three.product_key, self.three.device_name
            ),
            format!(
                "/sys/{}/{}/thing/file/upload/mqtt/cancel_reply",
                self.three.product_key, self.three.device_name
            ),
        ];
        for topic in topics {
            self.client.subscribe(topic, QoS::AtMostOnce).await?;
        }
        Ok(())
    }

    pub async fn upload(&self, path: impl AsRef<std::path::Path>) -> Result<String> {
        use tokio::fs::File;
        use tokio::io::AsyncReadExt;

        let params = InitParams {
            file_name: filename_for_path(&path)?,
            file_size: -1,
            conflict_strategy: Some(ConflictStrategy::Overwrite),
            ..Default::default()
        };
        let info = self.upload_init(params).await?;
        log::info!("upload_init: {:?}", info);

        let mut file = File::open(path).await?;

        let mut buf = Vec::with_capacity(4096);
        let mut offset = 0;
        let mut is_complete = Some(false);
        loop {
            let b_size = file.read(&mut buf).await?;
            if b_size == 0 {
                is_complete = Some(true);
            }
            let params = SendHeaderParams {
                upload_id: info.upload_id.clone(),
                offset,
                b_size,
                is_complete,
            };
            if let Err(err) = self.upload_send(params.clone(), buf[..b_size].into()).await {
                log::error!("upload_send: {:?}", err);
                self.upload_send(params, buf[..b_size].into()).await?;
            }
            if b_size == 0 {
                break;
            }
        }
        Ok(info.file_name)
    }
}

impl MqttConnection {
    pub fn file_uploader(&mut self) -> Result<Module> {
        let (tx, rx) = mpsc::channel(64);
        let (tx_, _rx_) = broadcast::channel(64);
        let executor = Executor {
            tx,
            tx_: tx_.clone(),
            three: self.mqtt_client.three.clone(),
        };
        self.module(Box::new(executor), rx, tx_)
    }
}

pub struct Executor {
    three: Arc<ThreeTuple>,
    tx: Sender<Recv>,
    tx_: broadcast::Sender<Recv>,
}

#[async_trait::async_trait]
impl crate::Executor for Executor {
    async fn execute(&self, topic: &str, payload: &[u8]) -> crate::Result<()> {
        let data = crate::execute::<RecvKind>(&self.three, topic, payload)?;
        self.tx_
            .send(data.clone())
            .map_err(|_| Error::BroadcastSendError)?;
        self.tx.send(data).await.map_err(|_| Error::MpscSendError)
    }
}
