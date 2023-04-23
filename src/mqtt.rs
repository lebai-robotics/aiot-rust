//! MQTT 协议接入。

use crate::util::auth;
use crate::*;
use log::*;
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, Transport};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum MqttInstance {
    Public(MqttPublicInstance),
    EndPoint(String),
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
            Self::EndPoint(url) => (url.to_string(), 443),
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
    pub three: Arc<ThreeTuple>,
    pub options: MqttOptions,
    pub(crate) executors: Vec<Box<dyn Executor + Send + Sync>>,
}

impl MqttClient {
    pub fn new(three: &ThreeTuple, info: &DeviceAuthInfo, instance: &MqttInstance) -> Result<Self> {
        let (host, port) = instance.url();
        let mut options = MqttOptions::new(&info.client_id, &host, port);
        options.set_credentials(&info.username, &info.password);

        Ok(Self {
            three: Arc::new(three.clone()),
            options,
            executors: Vec::new(),
        })
    }

    pub fn new_public(host: &str, three: &ThreeTuple) -> Result<Self> {
        let info = DeviceAuthInfo::from_tuple(three);
        let instance = MqttInstance::public(host, &three.product_key);
        Self::new(three, &info, &instance)
    }

    pub fn new_public_tls(host: &str, three: &ThreeTuple) -> Result<Self> {
        let mut res = Self::new_public(host, three)?;
        res.enable_tls()?;
        Ok(res)
    }

    pub fn new_tls(end_point: &str, three: &ThreeTuple) -> Result<Self> {
        let info = DeviceAuthInfo::from_tuple(three);
        let instance = MqttInstance::EndPoint(end_point.to_string());
        let mut res = Self::new(three, &info, &instance)?;
        res.enable_tls()?;
        Ok(res)
    }

    pub fn enable_tls(&mut self) -> Result<()> {
        let tls = auth::aliyun_client_config()?;
        self.options
            .set_transport(Transport::tls_with_config(tls.into()));
        Ok(())
    }

    pub fn connect(self) -> MqttConnection {
        MqttConnection::new(self)
    }
}

pub struct MqttConnection {
    pub event_loop: EventLoop,
    pub mqtt: Arc<AsyncClient>,
    pub mqtt_client: MqttClient,
}

impl MqttConnection {
    pub fn new(mqtt_client: MqttClient) -> Self {
        let options = mqtt_client.options.clone();
        let (mqtt, event_loop) = AsyncClient::new(options, 16);
        Self {
            mqtt_client,
            event_loop,
            mqtt: Arc::from(mqtt),
        }
    }
    pub async fn poll(&mut self) -> Result<Event> {
        let incoming = self.event_loop.poll().await?;
        match &incoming {
            Event::Incoming(packet) => match packet {
                Packet::Publish(data) => {
                    for e in &mut self.mqtt_client.executors {
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
            secure_mode,
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
