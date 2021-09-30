use crate::alink::{global_id_next, SysAck, ALINK_VERSION};
use crate::shadow::base::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

impl super::Module {
    /// 影子设备属性更新
    ///
    /// # 参数
    ///
    /// * `value` - 表示设备发送给设备影子的状态信息。reported为必填字段，状态信息会同步更新到设备影子的reported部分。
    /// * `version` - 表示设备影子检查请求中的版本信息。只有当新版本大于当前版本时，设备影子才会接收设备端的请求，并更新设备影子版本。如果version设置为-1时，表示清空设备影子数据，设备影子会接收设备端的请求，并将设备影子版本更新为0。
    pub async fn update(&self, value: Value, version: u32) -> crate::Result<()> {
        let payload = ShadowUpdateRequest {
            method: "update".to_string(),
            state: Some(value),
            version: Some(version),
        };
        self
            .publish(
                format!(
                    "/shadow/update/{}/{}",
                    self.three.product_key, self.three.device_name
                ),
                &payload,
            )
            .await
    }
    /// 影子设备属性获取
    pub async fn get(&self) -> crate::Result<()> {
        let payload = ShadowUpdateRequest {
            method: "get".to_string(),
            state: None,
            version: None,
        };
        self
            .publish(
                format!(
                    "/shadow/update/{}/{}",
                    self.three.product_key, self.three.device_name
                ),
                &payload,
            )
            .await
    }
    /// 影子设备属性删除
    ///
    /// # 参数
    ///
    /// * `value` 要删除的状态信息
    /// * `version` 版本
    pub async fn delete(&self, value: Value, version: u32) -> crate::Result<()> {
        let payload = ShadowUpdateRequest {
            method: "delete".to_string(),
            state: Some(value),
            version: Some(version),
        };
        self
            .publish(
                format!(
                    "/shadow/update/{}/{}",
                    self.three.product_key, self.three.device_name
                ),
                &payload,
            )
            .await
    }
}

// 影子设备更新
// /shadow/update/${YourProductKey}/${YourDeviceName}
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShadowUpdateRequest {
    pub method: String,
    pub state: Option<Value>,
    pub version: Option<u32>,
}
