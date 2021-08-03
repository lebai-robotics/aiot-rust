use super::recv::{DataModelRecv, RecvEnum};
use crate::alink::AlinkRequest;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DataModelMsg {
    /// 消息所属设备的product_key, 若为NULL则使用通过aiot_dm_setopt配置的product_key
    /// 在网关子设备场景下, 可通过指定为子设备的product_key来发送子设备的消息到云端
    pub product_key: Option<String>,
    /// 消息所属设备的device_name, 若为NULL则使用通过aiot_dm_setopt配置的device_name
    /// 在网关子设备场景下, 可通过指定为子设备的product_key来发送子设备的消息到云端
    pub device_name: Option<String>,
    /// 消息数据
    pub data: MsgEnum,
}

/// <b>物模型属性上报</b>消息结构体
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PropertyPost {
    /// 字符串形式的JSON结构体, <b>必须以结束符'\0'结尾</b>. 包含用户要上报的属性数据, 如<i>"{\"LightSwitch\":0}"</i>
    pub params: Value,
}

/// <b>物模型事件上报</b>消息结构体
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct EventPost {
    /// 事件标示符, <b>必须为以结束符'\0'结尾的字符串</b>
    pub event_id: String,
    /// 字符串形式的JSON结构体, <b>必须以结束符'\0'结尾</b>. 包含用户要上报的事件数据, 如<i>"{\"ErrorNum\":0}"</i>
    pub params: Map<String, Value>,
}

/// <b>属性设置应答</b>消息结构体, 用户在收到@ref AIOT_DMRECV_PROPERTY_SET 类型的属性设置后, 可发送此消息进行回复
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PropertySetReply {
    /// 消息标识符, uint64_t类型的整数, <b>必须与属性设置的消息标示符一致</b>
    pub msg_id: u64,
    /// 设备端状态码, 200-请求成功, 更多状态码查看<a href="https://help.aliyun.com/document_detail/89309.html">设备端通用code</a>
    pub code: u32,
    /// 设备端应答数据, 字符串形式的JSON结构体, <b>必须以结束符'\0'结尾</b>, 如<i>"{}"</i>表示应答数据为空
    pub data: String,
}

/// <b>异步服务应答</b>消息结构体, 用户在收到@ref AIOT_DMRECV_ASYNC_SERVICE_INVOKE 类型的异步服务调用消息后, 应发送此消息进行应答
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SyncServiceReply {
    /// 消息标识符, uint64_t类型的整数, <b>必须与同步服务调用的消息标示符一致</b>
    pub msg_id: u64,
    /// RRPC标示符, 用于唯一标识每一个同步服务的字符串, <b>必须与同步服务调用消息的RRPC标示符一致</b>
    pub rrpc_id: String,
    /// 服务标示符, 标识了要响应服务
    pub service_id: String,
    /// 设备端状态码, 200-请求成功, 更多状态码查看<a href="https://help.aliyun.com/document_detail/89309.html">设备端通用code</a>
    pub code: u32,
    /// 设备端应答数据, 字符串形式的JSON结构体, <b>必须以结束符'\0'结尾</b>, 如<i>"{}"</i>表示应答数据为空
    pub data: String,
}

/// <b>异步服务应答</b>消息结构体, 用户在收到@ref AIOT_DMRECV_ASYNC_SERVICE_INVOKE 类型的异步服务调用消息后, 应发送此消息进行应答
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AsyncServiceReply {
    /// 消息标识符, uint64_t类型的整数, <b>必须与异步服务调用的消息标示符一致</b>
    pub msg_id: u64,
    /// 服务标示符, 标识了要响应服务
    pub service_id: String,
    /// 设备端状态码, 200-请求成功, 更多状态码查看<a href="https://help.aliyun.com/document_detail/89309.html">设备端通用code</a>
    pub code: u32,
    /// 设备端应答数据, 字符串形式的JSON结构体, <b>必须以结束符'\0'结尾</b>, 如<i>"{}"</i>表示应答数据为空
    pub data: String,
}

/// <b>物模型二进制数据</b>消息结构体, 发送的二进制数据将通过物联网平台的JavaScript脚本转化为JSON格式数据, 用户发送此消息前应确保已正确启用云端解析脚本
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RawData {
    /// 二进制数据
    pub data: Vec<u8>,
}

/// <b>二进制格式的同步服务应答</b>消息结构体, 用户在收到@ref AIOT_DMRECV_RAW_SYNC_SERVICE_INVOKE 类型消息后, 应在超时时间(默认7s)内进行应答\n
/// 用户在使用此消息前应确保已启用云端解析脚本, 并且脚本工作正常
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RawServiceReply {
    /// RRPC标示符, 特殊字符串, <b>必须与同步服务调用消息的RRPC标示符一致</b>
    pub rrpc_id: String,
    /// 二进制数据
    pub data: Vec<u8>,
}

/// <b>获取期望属性值</b>消息结构体, 发送
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GetDesired {
    /// 字符串形式的JSON<b>数组</b>, <b>必须以结束符'\0'结尾</b>. 应包含用户要获取的期望属性的ID, 如<i>"[\"LightSwitch\"]"</i>
    pub params: Vec<String>,
}

/// <b>删除指定期望值</b>消息结构体
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeleteDesired {
    /// 字符串形式的JSON结构体, <b>必须以结束符'\0'结尾</b>. 应包含用户要删除的期望属性的ID和期望值版本号, 如<i>"{\"LightSwitch\":{\"version\":1},\"Color\":{}}"</i>
    pub params: String,
}

/// <b>物模型属性上报</b>消息结构体
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PropertyBatchPost {
    /**
     * @brief 字符串形式的JSON结构体, <b>必须以结束符'\0'结尾</b>. 包含用户要批量上报的属性和事件数据, 如 {"properties":{"Power": [ { "value": "on", "time": 1524448722000 },
     *  { "value": "off", "time": 1524448722001 } ], "WF": [ { "value": 3, "time": 1524448722000 }]}, "events": {"alarmEvent": [{ "value": { "Power": "on", "WF": "2"},
     *  "time": 1524448722000}]}}
     */
    params: String,
}

/// data-model模块发送消息类型
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum MsgEnum {
    /// 属性上报 成功发送此消息后, 将会收到@ref AIOT_DMRECV_GENERIC_REPLY 类型的应答消息
    PropertyPost(PropertyPost),
    /// 事件上报 成功发送此消息后, 将会收到@ref AIOT_DMRECV_GENERIC_REPLY 类型的应答消息
    EventPost(EventPost),
    /// 属性设置应答
    PropertySetReply(PropertySetReply),
    /// 异步服务应答
    AsyncServiceReply(AsyncServiceReply),
    /// 同步服务应答
    SyncServiceReply(SyncServiceReply),
    /// 二进制格式的物模型上行数据
    RawData(RawData),
    /// 二进制格式的同步服务应答
    RawServiceReply(RawServiceReply),
    /// 获取期望属性值 成功发送此消息后, 将会收到@ref AIOT_DMRECV_GENERIC_REPLY 类型的应答消息
    GetDesired(GetDesired),
    /// 清除指定的期望值 成功发送此消息后, 将会收到@ref AIOT_DMRECV_GENERIC_REPLY 类型的应答消息
    DeleteDesired(DeleteDesired),
    /// 清除指定的期望值 成功发送此消息后, 将会收到@ref AIOT_DMRECV_GENERIC_REPLY 类型的应答消息
    PropertyBatchPost(PropertyBatchPost),
}

impl DataModelMsg {
    pub fn new(data: MsgEnum) -> Self {
        Self {
            product_key: None,
            device_name: None,
            data,
        }
    }

    pub fn to_requset(&self, ack: i32) -> (String, AlinkRequest) {
        let pk = self.product_key.as_deref().unwrap_or("");
        let dn = self.device_name.as_deref().unwrap_or("");
        self.data.to_requset(&pk, &dn, ack)
    }
}

impl MsgEnum {
    pub fn new_property_post(params: Value) -> Self {
        MsgEnum::PropertyPost(PropertyPost { params })
    }

    pub fn to_requset(
        &self,
        product_key: &str,
        device_name: &str,
        ack: i32,
    ) -> (String, AlinkRequest) {
        use MsgEnum::*;
        let (method, topic, data) = match &self {
            PropertyPost(data) => (
                "thing.event.property.post",
                format!(
                    "/sys/{}/{}/thing/event/property/post",
                    product_key, device_name
                ),
                data.params.clone(),
            ),
            _ => ("", format!(""), Value::Null),
        };
        (topic, AlinkRequest::new(method, data, ack))
    }
}
