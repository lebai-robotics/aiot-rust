/// 设备描述信息三元组
#[derive(Debug, Clone, Default)]
pub struct ThreeTuple {
    pub product_key: String,
    pub device_name: String,
    pub device_secret: String,
}

impl ThreeTuple {
    pub fn from_env() -> Self {
        Self {
            product_key: std::env::var("PRODUCT_KEY").unwrap(),
            device_name: std::env::var("DEVICE_NAME").unwrap(),
            device_secret: std::env::var("DEVICE_SECRET").unwrap(),
        }
    }
}

pub const TOPICS: &'static [&str] = &[
    "/sys/+/+/thing/event/+/post_reply",
    "/sys/+/+/thing/service/property/set",
    "/sys/+/+/thing/service/+",
    "/ext/rrpc/+/sys/+/+/thing/service/+",
    "/sys/+/+/thing/model/down_raw",
    "/sys/+/+/thing/model/up_raw_reply",
    "/ext/rrpc/+/sys/+/+/thing/model/down_raw",
    "/sys/+/+/thing/property/desired/get_reply",
    "/sys/+/+/thing/property/desired/delete_reply",
    "/sys/+/+/thing/event/property/batch/post_reply",
];
