use std::fs;

use crypto::digest::Digest;
use log::{debug};
use tempdir::TempDir;
use crate::Error;

use crate::alink::{global_id_next, SysAck, ALINK_VERSION};
use crate::http_downloader::{HttpDownloadConfig, HttpDownloader};
use crate::ota::push_dto::*;

use super::Runner;
use super::recv_dto::UpgradePackageRequest;

impl Runner {
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
   
   pub async fn receive_upgrade_package(
      &mut self,
      package: &UpgradePackageRequest,
   ) -> crate::Result<String> {
      debug!("start receive_upgrade_package");
      let module = package.data.module.clone();
      let version = package.data.version.clone();
      let tmp_dir = TempDir::new("ota")?;
      let file_path = tmp_dir.path().join(format!(
         "{}_{}",
         module.clone().unwrap_or(String::from("default")),
         version
      ));
      let downloader = HttpDownloader::new(HttpDownloadConfig {
         block_size: 8000000,
         uri: package.data.url.clone(),
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
      debug!("size:{}", buffer.len());
      match package.data.sign_method.as_str() {
         "SHA256" => {
            let mut sha256 = crypto::sha2::Sha256::new();
            sha256.input(&buffer);
            let computed_result = sha256.result_str();
            if computed_result != package.data.sign {
               debug!(
                  "computed_result:{} sign:{}",
                  computed_result, package.data.sign
               );
               return Err(Error::FileValidateFailed);
            }
         }
         "Md5" => {
            let mut md5 = crypto::md5::Md5::new();
            md5.input(&buffer);
            let computed_result = md5.result_str();
            if computed_result != package.data.sign {
               debug!(
                  "computed_result:{} sign:{}",
                  computed_result, package.data.sign
               );
               return Err(Error::FileValidateFailed);
            }
         }
         _ => {
            return Err(Error::FileValidateFailed);
         }
      }

      std::fs::remove_file(file_path);
      std::fs::remove_dir_all(tmp_dir);
      debug!("receive_upgrade_package finished");
      Ok(ota_file_path)
   }
}
