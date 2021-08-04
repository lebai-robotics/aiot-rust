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
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AlinkRequest {
    pub id: String,
    pub version: String,
    pub params: Value,
    pub sys: SysAck,
    pub method: String,
}

impl AlinkRequest {
    pub fn new(method: &str, params: Value, ack: i32) -> Self {
        Self {
            id: format!("{}", global_id_next()),
            version: "1.0".to_string(),
            params,
            sys: SysAck { ack },
            method: method.to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SysAck {
    pub ack: i32,
}
