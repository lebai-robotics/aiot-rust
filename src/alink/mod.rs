use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicI32, Ordering};

static ID: AtomicI32 = AtomicI32::new(1);

pub fn global_id_next() -> i32 {
    ID.fetch_add(1, Ordering::SeqCst)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct AlinkResponse {
    id: String,
    code: i32,
    data: Value,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AlinkRequest {
    id: String,
    version: String,
    params: Value,
    sys: SysAck,
    method: String,
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
struct SysAck {
    ack: i32,
}
