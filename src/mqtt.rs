use crate::util::auth;
use crate::*;
use log::*;
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, Transport};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum MqttInstance {
    Public(MqttPublicInstance),
    Enterprise(String),
}

impl MqttInstance {
    pub fn public(host: &str, product_key: &str) -> Self {
        Self::Public(MqttPublicInstance {
            host: host.to_string(),
            port: 443,
            product_key: product_key.to_string(),
        })
    }

    pub fn url(&self) -> (String, u16) {
        match self {
            Self::Public(p) => (format!("{}.{}", p.product_key, p.host), p.port),
            Self::Enterprise(url) => (url.to_string(), 443),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MqttPublicInstance {
    host: String,
    port: u16,
    product_key: String,
}

pub struct MqttClient {
    pub three: ThreeTuple,
    pub options: MqttOptions,
    pub(crate) executors: Vec<Box<dyn Executor>>,
}

impl MqttClient {
    pub fn new(three: &ThreeTuple, info: &DeviceAuthInfo, instance: &MqttInstance) -> Result<Self> {
        let (host, port) = instance.url();
        let mut options = MqttOptions::new(&info.client_id, &host, port);
        options.set_keep_alive(300);
        options.set_credentials(&info.username, &info.password);

        Ok(Self {
            three: three.clone(),
            options,
            executors: Vec::new(),
        })
    }

    pub fn new_public(host: &str, three: &ThreeTuple) -> Result<Self> {
        let info = DeviceAuthInfo::from_tuple(&three);
        let instance = MqttInstance::public(&host, &three.product_key);
        Self::new(three, &info, &instance)
    }

    pub fn new_public_tls(host: &str, three: &ThreeTuple) -> Result<Self> {
        let mut res = Self::new_public(&host, &three)?;
        res.enable_tls()?;
        Ok(res)
    }

    pub fn enable_tls(&mut self) -> Result<()> {
        let tls = auth::aliyun_client_config()?;
        self.options
            .set_transport(Transport::tls_with_config(tls.into()));
        Ok(())
    }

    pub fn connect(self) -> (AsyncClient, MqttEventLoop) {
        let (mqtt, eventloop) = AsyncClient::new(self.options, 16);
        let el = MqttEventLoop {
            eventloop,
            executors: self.executors,
        };
        (mqtt, el)
    }
}

pub struct MqttEventLoop {
    eventloop: EventLoop,
    executors: Vec<Box<dyn Executor>>,
}

impl MqttEventLoop {
    pub async fn poll(&mut self) -> Result<Event> {
        let incoming = self.eventloop.poll().await?;
        match &incoming {
            Event::Incoming(packet) => match packet {
                Packet::Publish(data) => {
                    for e in &self.executors {
                        if let Err(err) = e.execute(&data.topic, &data.payload).await {
                            debug!("{} error: {}", data.topic, err);
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
        Ok(incoming)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeviceAuthInfo {
    pub client_id: String,
    pub username: String,
    pub password: String,
}

impl DeviceAuthInfo {
    pub fn from_tuple(three: &ThreeTuple) -> Self {
        let username = auth::mqtt::username(&three.product_key, &three.device_name);
        let password = auth::mqtt::password(
            &three.product_key,
            &three.device_name,
            &three.device_secret,
            false,
        );
        let secure_mode = "2";
        let client_id = auth::mqtt::client_id(
            &three.product_key,
            &three.device_name,
            &secure_mode,
            "",
            false,
        );
        Self {
            client_id,
            username,
            password,
        }
    }
}
