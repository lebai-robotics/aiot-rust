use crate::util::{auth, rand_string};
use crate::{DeviceAuthInfo, Error, MqttClient, MqttInstance, Result};
use log::*;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

pub struct DynamicRegister {
    mqtt: MqttClient,
    rx: mpsc::Receiver<DynamicRegisterResult>,
}

impl DynamicRegister {
    pub fn new_public_tls(
        host: &str,
        product_key: &str,
        product_secret: &str,
        device_name: &str,
    ) -> Result<Self> {
        let (tx, rx) = mpsc::channel(4);
        let rego = Box::new(DynamicRegisterOptions::new(
            &product_key,
            &product_secret,
            &device_name,
            tx,
        ));
        let username = rego.username();
        let password = rego.password();
        let client_id = rego.client_id();
        let info = DeviceAuthInfo {
            username,
            password,
            client_id,
        };
        let instance = MqttInstance::public(&host, &product_key);
        let mut mqtt = MqttClient::new(&info, &instance)?;
        mqtt.enable_tls()?;
        mqtt.executors.push(rego as Box<dyn crate::Executor>);
        Ok(Self { mqtt, rx })
    }

    pub async fn register(mut self) -> Result<DynamicRegisterResult> {
        let (_, mut eventloop) = self.mqtt.connect();
        loop {
            tokio::select! {
                Some(res) = self.rx.recv() => {
                    return Ok(res);
                },
                Ok(n) = eventloop.poll() => {
                    debug!("Received = {:?}", n);
                },
                else => {
                    return Err(Error::EventLoopError);
                }
            }
        }
    }
}

pub struct DynamicRegisterOptions {
    random: String,
    pub product_key: String,
    pub product_secret: String,
    pub device_name: String,
    pub no_whitelist: bool,
    pub instance_id: Option<String>,
    pub tx: mpsc::Sender<DynamicRegisterResult>,
}

impl DynamicRegisterOptions {
    pub fn new(
        product_key: &str,
        product_secret: &str,
        device_name: &str,
        tx: mpsc::Sender<DynamicRegisterResult>,
    ) -> Self {
        Self {
            product_key: product_key.to_string(),
            product_secret: product_secret.to_string(),
            device_name: device_name.to_string(),
            random: rand_string(4),
            no_whitelist: false,
            instance_id: None,
            tx,
        }
    }

    pub fn username(&self) -> String {
        auth::mqtt::username(&self.product_key, &self.device_name)
    }

    pub fn password(&self) -> String {
        let input = format!(
            "deviceName{}productKey{}random{}",
            self.device_name, self.product_key, self.random
        );
        auth::sign(&input, &self.product_secret)
    }

    pub fn client_id(&self) -> String {
        let auth_type = if self.no_whitelist {
            "regnwl"
        } else {
            "register"
        };
        let instance = if let Some(id) = &self.instance_id {
            format!(",instanceId={}", id)
        } else {
            "".to_string()
        };
        format!(
            "{}.{}|random={},authType={},securemode={},signmethod={}{}|",
            self.device_name,
            self.product_key,
            self.random,
            auth_type,
            2,
            auth::SIGN_METHOD,
            instance
        )
    }
}

#[async_trait::async_trait]
impl crate::Executor for DynamicRegisterOptions {
    async fn execute(&self, topic: &str, payload: &[u8]) -> Result<()> {
        if topic == "/ext/register" {
            let data: DeviceInfoWhitelist = serde_json::from_slice(&payload)?;
            self.tx
                .send(DynamicRegisterResult::Whitelist(data))
                .await
                .map_err(|_| Error::RepeatRegisterResponse)?;
            Ok(())
        } else if topic == "/ext/regnwl" {
            let data: RecvNoWhitelist = serde_json::from_slice(&payload)?;
            let conn_clientid = format!(
                "{}|authType=connwl,securemode=-2,_ss=1,ext=3,_v={}|",
                data.client_id,
                auth::CORE_AUTH_SDK_VERSION
            );
            let username = self.username();
            let res = DynamicRegisterResult::NoWhitelist(DeviceAuthInfo {
                client_id: conn_clientid,
                username,
                password: data.device_token,
            });
            self.tx
                .send(res)
                .await
                .map_err(|_| Error::RepeatRegisterResponse)?;
            Ok(())
        } else {
            Err(Error::InvalidTopic(topic.to_string()))
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeviceInfoWhitelist {
    #[serde(rename = "deviceSecret")]
    pub device_secret: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RecvNoWhitelist {
    #[serde(rename = "clientId")]
    pub client_id: String,
    #[serde(rename = "deviceToken")]
    pub device_token: String,
}

#[derive(Debug, Clone)]
pub enum DynamicRegisterResult {
    Whitelist(DeviceInfoWhitelist),
    NoWhitelist(DeviceAuthInfo),
}
