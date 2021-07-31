#![allow(dead_code)]

pub mod util;
pub use util::error::{Error, Result};

mod mqtt;
pub use mqtt::{DeviceAuthInfo, MqttClient, MqttInstance};

mod ra;
pub use ra::{RemoteAccess, Runner as RemoteAccessRunner, RemoteAccessTrait};

mod dm;
pub use dm::ThreeTuple;

mod dynregmq;
pub use dynregmq::{DynamicRegister, DynamicRegisterResult};

#[async_trait::async_trait]
pub trait Executor {
    async fn execute(&self, topic: &str, payload: &[u8]) -> Result<()>;
}
