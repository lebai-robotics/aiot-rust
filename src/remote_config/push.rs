use super::recv::RemoteConfigFileInfo;
use crate::Error;
use crate::{
    alink::{global_id_next, AlinkRequest, AlinkResponse, SysAck, ALINK_VERSION},
    http_downloader::{HttpDownloadConfig, HttpDownloader},
};
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use tempdir::TempDir;

impl super::Module {
    /// 设备主动请求配置信息
    pub async fn get(&self, ack: bool) -> crate::Result<()> {
        let payload = RemoteConfigGetRequest {
            id: global_id_next().to_string(),
            params: RemoteConfigGetParams {
                config_scope: "product".to_string(),
                get_type: "file".to_string(),
            },
            version: ALINK_VERSION.to_string(),
            sys: Some(SysAck { ack: ack.into() }),
            method: None,
        };
        self.publish(
            format!(
                "/sys/{}/{}/thing/config/get",
                self.three.product_key, self.three.device_name
            ),
            &payload,
        )
        .await
    }
    /// 配置推送回应
    pub async fn push_reply(&self, id: String, code: u64) -> crate::Result<()> {
        let payload = AlinkResponse {
            id,
            code,
            data: (),
            message: None,
            method: None,
            version: None,
        };
        self.publish(
            format!(
                "/sys/{}/{}/thing/config/push_reply",
                self.three.product_key, self.three.device_name
            ),
            &payload,
        )
        .await
    }

    /// 下载配置直到完成，返回二进制数据
    ///
    /// # 参数
    ///
    /// * `config_info` - 配置信息
    pub async fn download_config(
        &mut self,
        config_info: &RemoteConfigFileInfo,
    ) -> crate::Result<Vec<u8>> {
        let config_id = config_info.config_id.clone();
        let tmp_dir = TempDir::new("remote_config")?;
        let file_path = tmp_dir.path().join(config_id.to_string());
        let downloader = HttpDownloader::new(&config_info.url, file_path);
        let config_file_path = downloader.start().await?;
        let mut buffer = fs::read(config_file_path)?;
        crate::util::validate(&buffer, &config_info.sign_method, &config_info.sign)?;
        // std::fs::remove_file(file_path);
        // std::fs::remove_dir_all(tmp_dir);
        Ok(buffer)
    }

    /* 	/// 设备主动上报网络状态
    pub async fn report_network_status(&self)-> crate::Result<()>{
        let payload = AlinkResponse {
            id,
            code,
            data: (),
            message: None,
            method: None,
            version: None,
        };
        self
            .publish(
                format!(
                    "/sys/{}/{}/_thing/diag/post",
                    self.three.product_key, self.three.device_name
                ),
                &payload,
            )
            .await
    } */
}

/// 远程配置获取请求
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemoteConfigGetParams {
    /// 配置范围， 目前只支持产品维度配置。 取值：product。
    pub config_scope: String,
    /// get_type
    /// 获取配置类型。 目前支持文件类型，取值：file。
    pub get_type: String,
}

pub type RemoteConfigGetRequest = AlinkRequest<RemoteConfigGetParams>;
