use crate::Error;
use crate::subdev::recv_dto::*;
use log::{debug};

#[async_trait::async_trait]
impl crate::Executor for crate::subdev::Executor {
	async fn execute(&self, topic: &str, payload: &[u8]) -> crate::Result<()> {
		debug!("receive: {} {}", topic, String::from_utf8_lossy(payload));
		if let Some(kind) = SubDevRecvKind::match_kind(topic, &self.three.product_key, &self.three.device_name){
			let data = kind.to_payload(payload)?;
			self.tx.send(data).await.map_err(|_| Error::MpscSendError)?;
		}
		debug!("no match topic: {}", topic);
		Ok(())
	}
}