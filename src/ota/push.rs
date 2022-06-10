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
    /// 设备上报OTA模块版本
    ///
    /// # 参数
    ///
    /// * `version` - 上报的版本
    /// * `module` - 上报的OTA模块，默认为 "default"
    pub async fn report_version(
        &mut self,
        version: String,
        module: Option<String>,
    ) -> crate::Result<()> {
        let payload = ReportVersionRequest {
            id: global_id_next().to_string(),
            params: ReportVersion { version, module },
            version: ALINK_VERSION.to_string(),
            sys: None,
            method: None,
        };
        let topic = format!(
            "/ota/device/inform/{}/{}",
            self.three.product_key, self.three.device_name
        );
        self.publish(topic, &payload).await;
        Ok(())
    }

    /// 设备上报升级进度
    pub async fn report_process(&mut self, params: ReportProgress) -> crate::Result<()> {
        let payload = ReportProgressRequest {
            id: global_id_next().to_string(),
            params,
            version: ALINK_VERSION.to_string(),
            sys: None,
            method: None,
        };
        let topic = format!(
            "/ota/device/progress/{}/{}",
            self.three.product_key, self.three.device_name
        );
        self.publish(topic, &payload).await;
        Ok(())
    }

    /// 设备请求升级包信息
    ///
    /// # 参数
    ///
    /// * `module` - 请求的OTA模块，默认为 "default"
    pub async fn query_firmware(&mut self, module: Option<String>) -> crate::Result<()> {
        let payload = QueryFirmwareRequest {
            id: global_id_next().to_string(),
            params: QueryFirmware { module },
            version: ALINK_VERSION.to_string(),
            sys: None,
            method: None,
        };
        let topic = format!(
            "/sys/{}/{}/thing/ota/firmware/get",
            self.three.product_key, self.three.device_name
        );
        self.publish(topic, &payload).await;
        Ok(())
    }

    /// 下载升级包到文件
    pub async fn download_to(
        &mut self,
        package: &PackageData,
        path: impl AsRef<Path>,
    ) -> crate::Result<String> {
        let module = package.module.clone();
        let version = package.version.clone();
        let downloader = HttpDownloader::new(&package.url, path);
        let results = futures_util::future::join(
            async {
                let process_receiver = downloader.get_process_receiver();
                let mut mutex_guard = process_receiver.lock().await;
                if let Some(download_process) = mutex_guard.recv().await {
                    let report_progress = ReportProgress {
                        module: module.clone(),
                        desc: String::from(""),
                        step: ((download_process.percent * 100f64) as u32).to_string(),
                    };
                    debug!("report_process finished {}", report_progress.step);
                    self.report_process(report_progress);
                }
            },
            downloader.start(),
        )
        .await;
        let mut ota_file_path = results.1?;
        let mut buffer = fs::read(&ota_file_path)?;
        crate::util::validate(&buffer, &package.sign_method, &package.sign)?;
        Ok(ota_file_path)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReportVersion {
    pub version: String,
    pub module: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReportProgress {
    pub step: String,
    pub desc: String,
    pub module: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct QueryFirmware {
    pub module: Option<String>,
}

pub type ReportVersionRequest = AlinkRequest<ReportVersion>;
pub type ReportProgressRequest = AlinkRequest<ReportProgress>;
pub type QueryFirmwareRequest = AlinkRequest<QueryFirmware>;
