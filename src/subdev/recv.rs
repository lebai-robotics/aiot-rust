use serde_json::Value;
use std::collections::HashMap;
use std::any::TypeId;
use crate::Error;
use crate::subdev::recv_dto::*;
use log::*;

#[async_trait::async_trait]
impl crate::Executor for crate::subdev::Executor {
	async fn execute(&self, topic: &str, payload: &[u8]) -> crate::Result<()> {
		debug!("{} {}", topic, String::from_utf8_lossy(payload));
		for item in &*TOPICS {
			if !item.is_match(topic, &self.three.product_key, &self.three.device_name) {
				return Ok(());
			}
			let data = match item.payload_type_id {
				a if a == TypeId::of::<SubDevLoginResponse>() => {
					SubDevRecv::SubDevLoginResponse(serde_json::from_slice(&payload)?)
				}
				a if a == TypeId::of::<SubDevBatchLoginResponse>() => {
					SubDevRecv::SubDevBatchLoginResponse(serde_json::from_slice(&payload)?)
				}
				a if a == TypeId::of::<SubDevLogoutResponse>() => {
					SubDevRecv::SubDevLogoutResponse(serde_json::from_slice(&payload)?)
				}
				a if a == TypeId::of::<SubDevBatchLogoutResponse>() => {
					SubDevRecv::SubDevBatchLogoutResponse(serde_json::from_slice(&payload)?)
				}
				a if a == TypeId::of::<SubDevMethodResponse>() => {
					SubDevRecv::SubDevMethodResponse(serde_json::from_slice(&payload)?)
				}
				a if a == TypeId::of::<SubDevAddTopologicalRelationResponse>() => {
					SubDevRecv::SubDevAddTopologicalRelationResponse(serde_json::from_slice(&payload)?)
				}
				_ => {
					return Ok(());
				}
			};
			self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
		}
		Ok(())
	}
}