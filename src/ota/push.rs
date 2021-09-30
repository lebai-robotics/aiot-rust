use std::fs;

use crate::Error;
use crypto::digest::Digest;
use log::debug;
use tempdir::TempDir;

use crate::alink::{global_id_next, SysAck, ALINK_VERSION};
use crate::http_downloader::{HttpDownloadConfig, HttpDownloader};
use serde_json::Value;
use std::collections::HashMap;

use crate::alink::alink_topic::ALinkSubscribeTopic;
use crate::alink::{AlinkRequest, AlinkResponse};
use crate::subdev::base::DeviceInfoId;
use enum_iterator::IntoEnumIterator;
use enum_kinds::EnumKind;
use serde::{Deserialize, Serialize};

use super::base::*;

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
        self
            .publish(
                format!(
                    "/ota/device/inform/{}/{}",
                    self.three.product_key, self.three.device_name
                ),
                &payload,
            )
            .await;
        Ok(())
    }
    /// 设备上报升级进度
    pub async fn report_process(&mut self, report_process: ReportProgress) -> crate::Result<()> {
        let payload = ReportProgressRequest {
            id: global_id_next().to_string(),
            params: report_process,
            version: ALINK_VERSION.to_string(),
            sys: None,
            method: None,
        };
        self
            .publish(
                format!(
                    "/ota/device/progress/{}/{}",
                    self.three.product_key, self.three.device_name
                ),
                &payload,
            )
            .await;
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
        self
            .publish(
                format!(
                    "/sys/{}/{}/thing/ota/firmware/get",
                    self.three.product_key, self.three.device_name
                ),
                &payload,
            )
            .await;
        Ok(())
    }
    /// 下载升级包直到完成，返回二进制数据
    ///
    /// # 参数
    ///
    /// * `package` - 升级包信息
    pub async fn download_upgrade_package(
        &mut self,
        package: &PackageData,
    ) -> crate::Result<Vec<u8>> {
        debug!("start receive_upgrade_package");
        let module = package.module.clone();
        let version = package.version.clone();
        let tmp_dir = TempDir::new("ota")?;
        let file_path = tmp_dir.path().join(format!(
            "{}_{}",
            module.clone().unwrap_or(String::from("default")),
            version
        ));
        let downloader = HttpDownloader::new(HttpDownloadConfig {
            block_size: 8000000,
            uri: package.url.clone(),
            file_path: String::from(file_path.to_str().unwrap()),
        });
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
        let mut buffer = fs::read(ota_file_path.clone())?;
        debug!("file size:{}", buffer.len());
        match package.sign_method.as_str() {
            "SHA256" | "Sha256" => {
                let mut sha256 = crypto::sha2::Sha256::new();
                sha256.input(&buffer);
                let computed_result = sha256.result_str();
                if computed_result != package.sign {
                    debug!("computed_result:{} sign:{}", computed_result, package.sign);
                    return Err(Error::FileValidateFailed);
                }
            }
            "Md5" | "MD5" => {
                let mut md5 = crypto::md5::Md5::new();
                md5.input(&buffer);
                let computed_result = md5.result_str();
                if computed_result != package.sign {
                    debug!("computed_result:{} sign:{}", computed_result, package.sign);
                    return Err(Error::FileValidateFailed);
                }
            }
            _ => {
                return Err(Error::FileValidateFailed);
            }
        }
        Ok(buffer)
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
