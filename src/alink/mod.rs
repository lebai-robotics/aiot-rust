use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};

static ID: AtomicU64 = AtomicU64::new(1);

pub fn global_id_next() -> u64 {
    ID.fetch_add(1, Ordering::SeqCst)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AlinkResponse {
    pub id: String,
    pub code: u32,
    pub data: Value,
    pub message: Option<String>,
    pub method: Option<String>,
    pub version: Option<String>,
}

impl AlinkResponse {
    pub fn msg_id(&self) -> u64 {
        self.id.parse().unwrap_or(0)
    }

    pub fn new(id: u64, code: u32, data: Value) -> Self {
        Self {
            id: format!("{}", id),
            code,
            data,
            message: None,
            version: None,
            method: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AlinkRequest {
    pub id: String,
    pub version: String,
    pub params: Value,
    pub sys: Option<SysAck>,
    pub method: String,
}

impl AlinkRequest {
    pub fn msg_id(&self) -> u64 {
        self.id.parse().unwrap_or(0)
    }

    pub fn new_id(id: u64, method: &str, params: Value, ack: i32) -> Self {
        Self {
            id: format!("{}", id),
            version: "1.0".to_string(),
            params,
            sys: Some(SysAck { ack }),
            method: method.to_string(),
        }
    }

    pub fn new(method: &str, params: Value, ack: i32) -> Self {
        Self::new_id(global_id_next(), method, params, ack)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SysAck {
    pub ack: i32,
}
