use super::base::*;
use crate::alink::alink_topic::ALinkSubscribeTopic;
use crate::alink::{global_id_next, SysAck, ALINK_VERSION};
use crate::alink::{AlinkRequest, AlinkResponse};
use crate::http_downloader::{HttpDownloadConfig, HttpDownloader};
use crate::subdev::base::DeviceInfoId;
use crate::Error;
use enum_iterator::IntoEnumIterator;
use enum_kinds::EnumKind;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tempdir::TempDir;

impl super::Module {
    /// 设设备请求SSH远程通道认证信息
    pub async fn proxy_request(&mut self) -> crate::Result<()> {
        let payload = ();
        let topic = format!(
            "/sys/{}/{}/secure_tunnel/proxy/request",
            self.three.product_key, self.three.device_name
        );
        self.publish(topic, &payload).await;
        Ok(())
    }
}
