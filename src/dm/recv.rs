use serde::{Deserialize, Serialize};
use serde_json::Value;

/// data-model模块接收消息的结构体
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DataModelRecv {
    /// 消息所属设备的product_key, 不配置则默认使用MQTT模块配置的product_key
    pub product_key: String,
    /// 消息所属设备的device_name, 不配置则默认使用MQTT模块配置的device_name
    pub device_name: String,
    /// 接收消息数据
    pub data: RecvEnum,
}
/// <b>云端通用应答</b>消息结构体, 设备端上报@ref AIOT_DMMSG_PROPERTY_POST, @ref AIOT_DMMSG_EVENT_POST 或者@ref AIOT_DMMSG_GET_DESIRED 等消息后, 服务器会应答此消息
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GenericReply {
    /// 消息标识符, uint64_t类型的整数, 与属性上报或事件上报的消息标示符一致
    pub msg_id: u64,
    /// 设备端错误码, 200-请求成功, 更多错误码码查看<a href="https://help.aliyun.com/document_detail/120329.html">设备端错误码</a>
    pub code: u64,
    /// 云端应答数据
    pub data: Value,
    /// 状态消息字符串, 当设备端上报请求成功时对应的应答消息为"success", 若请求失败则应答消息中包含错误信息
    pub message: String,
}

impl DataModelRecv {
    pub fn generic_reply(pk: &str, dn: &str, data: GenericReply) -> Self {
        Self {
            product_key: pk.to_string(),
            device_name: dn.to_string(),
            data: RecvEnum::GenericReply(data),
        }
    }
}

/// <b>属性设置</b>消息结构体
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PropertySet {
    /// 消息标识符, uint64_t类型的整数
    pub msg_id: u64,
    /// 服务器下发的属性数据, 为字符串形式的JSON结构体, 此字符串<b>不</b>以结束符'\0'结尾, 如<i>"{\"LightSwitch\":0}"</i>
    pub params: Value,
}

impl DataModelRecv {
    pub fn property_set(pk: &str, dn: &str, data: PropertySet) -> Self {
        Self {
            product_key: pk.to_string(),
            device_name: dn.to_string(),
            data: RecvEnum::PropertySet(data),
        }
    }
}

/// <b>同步服务调用</b>消息结构体, 用户收到同步服务后, 必须在超时时间(默认7s)内进行应答
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SyncServiceInvoke {
    /// 消息标识符, uint64_t类型的整数
    pub msg_id: u64,
    /// RRPC标识符, 用于唯一标识每一个同步服务的特殊字符串
    pub rrpc_id: String,
    /// 服务标示符, 字符串内容由用户定义的物模型决定
    pub service_id: String,
    /// 服务调用的输入参数数据, 为字符串形式的JSON结构体, 此字符串<b>不</b>以结束符'\0'结尾, 如<i>"{\"LightSwitch\":0}"</i>
    pub params: Value,
}

impl DataModelRecv {
    pub fn sync_service_invoke(pk: &str, dn: &str, data: SyncServiceInvoke) -> Self {
        Self {
            product_key: pk.to_string(),
            device_name: dn.to_string(),
            data: RecvEnum::SyncServiceInvoke(data),
        }
    }
}

/// <b>异步服务调用</b>消息结构体
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AsyncServiceInvoke {
    /// 消息标识符, uint64_t类型的整数
    pub msg_id: u64,
    /// 服务标示符, 字符串内容由用户定义的物模型决定
    pub service_id: String,
    /// 服务调用的输入参数数据, 为字符串形式的JSON结构体, 此字符串<b>不</b>以结束符'\0'结尾, 如<i>"{\"LightSwitch\":0}"</i>
    pub params: Value,
}

impl DataModelRecv {
    pub fn async_service_invoke(pk: &str, dn: &str, data: AsyncServiceInvoke) -> Self {
        Self {
            product_key: pk.to_string(),
            device_name: dn.to_string(),
            data: RecvEnum::AsyncServiceInvoke(data),
        }
    }
}

/// <b>物模型二进制数据</b>消息结构体, 服务器的JSON格式物模型数据将通过物联网平台的JavaScript脚本转化为二进制数据, 用户在接收此消息前应确保已正确启用云端解析脚本
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RawData {
    /// 二进制数据
    pub data: Vec<u8>,
}

impl DataModelRecv {
    pub fn raw_data(pk: &str, dn: &str, data: RawData) -> Self {
        Self {
            product_key: pk.to_string(),
            device_name: dn.to_string(),
            data: RecvEnum::RawData(data),
        }
    }

    pub fn raw_data_reply(pk: &str, dn: &str, data: RawData) -> Self {
        Self {
            product_key: pk.to_string(),
            device_name: dn.to_string(),
            data: RecvEnum::RawDataReply(data),
        }
    }
}

/// <b>二进制数据的同步服务调用</b>消息结构体, 服务器的JSON格式物模型数据将通过物联网平台的JavaScript脚本转化为二进制数据, 用户在接收此消息前应确保已正确启用云端解析脚本
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RawServiceInvoke {
    /// RRPC标识符, 用于唯一标识每一个同步服务的特殊字符串
    pub rrpc_id: String,
    /// 二进制数据
    pub data: Vec<u8>,
}

impl DataModelRecv {
    pub fn raw_sync_service_invoke(pk: &str, dn: &str, data: RawServiceInvoke) -> Self {
        Self {
            product_key: pk.to_string(),
            device_name: dn.to_string(),
            data: RecvEnum::RawSyncServiceInvoke(data),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum RecvEnum {
    /// 上报属性/实践后服务器返回的应答消息
    GenericReply(GenericReply),
    /// 服务器下发的属性设置消息
    PropertySet(PropertySet),
    /// 服务器下发的异步服务调用消息
    AsyncServiceInvoke(AsyncServiceInvoke),
    /// 服务器下发的同步服务调用消息
    SyncServiceInvoke(SyncServiceInvoke),
    /// 服务器对设备上报的二进制数据应答
    RawDataReply(RawData),
    /// 服务器下发的物模型二进制数据
    RawData(RawData),
    /// 服务器下发的二进制格式的同步服务调用消息
    RawSyncServiceInvoke(RawServiceInvoke),
}
