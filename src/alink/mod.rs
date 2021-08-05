use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

static ID: AtomicU64 = AtomicU64::new(1);

pub fn global_id_next() -> u64 {
    ID.fetch_add(1, Ordering::SeqCst)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AlinkResponse<T> {
    pub id: String,
    pub code: u32,
    pub data: T,
    pub message: Option<String>,
    pub method: Option<String>,
    pub version: Option<String>,
}

impl<T> AlinkResponse<T> {
    pub fn msg_id(&self) -> u64 {
        self.id.parse().unwrap_or(0)
    }

    pub fn new(id: u64, code: u32, data: T) -> Self {
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
pub struct AlinkRequest<T> {
    pub id: String,
    pub version: String,
    pub params: T,
    pub sys: Option<SysAck>,
    pub method: Option<String>,
}

impl<T> AlinkRequest<T> {
    pub fn msg_id(&self) -> u64 {
        self.id.parse().unwrap_or(0)
    }

    pub fn new_id(id: u64, method: &str, params: T, ack: i32) -> Self {
        Self {
            id: format!("{}", id),
            version: "1.0".to_string(),
            params,
            sys: Some(SysAck { ack }),
            method: Some(method.to_string()),
        }
    }

    pub fn new(method: &str, params: T, ack: i32) -> Self {
        Self::new_id(global_id_next(), method, params, ack)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SysAck {
    pub ack: i32,
}
