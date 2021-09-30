use std::fs;

use crate::alink::{global_id_next, AlinkRequest, AlinkResponse, SysAck, ALINK_VERSION};
use crate::Error;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::Value;

impl super::Module {
    /// 设备分发通知响应
    pub async fn notify_reply(&self, id: String, code: u64) -> crate::Result<()> {
        let payload = AlinkResponse::<()> {
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
                    "/sys/{}/{}/thing/bootstrap/notify_reply",
                    self.three.product_key, self.three.device_name
                ),
                &payload,
            )
            .await
    }
}
