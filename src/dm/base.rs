use crate::alink::{AlinkRequest, AlinkResponse};
use crate::{Error, Result, ThreeTuple};
use log::*;
use regex::Regex;
use rumqttc::{AsyncClient, QoS};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

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
    /// 字符串形式的JSON结构体. 包含用户要上报的属性数据, 如<i>"{\"LightSwitch\":0}"</i>
    pub params: Value,
}

/// <b>物模型事件上报</b>消息结构体
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct EventPost {
    /// 事件标示符, <b>必须为以结束符'\0'结尾的字符串</b>
    pub event_id: String,
    /// 字符串形式的JSON结构体. 包含用户要上报的事件数据, 如<i>"{\"ErrorNum\":0}"</i>
    pub params: Value,
}

/// <b>属性设置应答</b>消息结构体, 用户在收到@ref AIOT_DMRECV_PROPERTY_SET 类型的属性设置后, 可发送此消息进行回复
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PropertySetReply {
    /// 消息标识符, uint64_t类型的整数, <b>必须与属性设置的消息标示符一致</b>
    pub msg_id: u64,
    /// 设备端状态码, 200-请求成功, 更多状态码查看<a href="https://help.aliyun.com/document_detail/89309.html">设备端通用code</a>
    pub code: u64,
    /// 设备端应答数据, 字符串形式的JSON结构体, 如<i>"{}"</i>表示应答数据为空
    pub data: Value,
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
    pub code: u64,
    /// 设备端应答数据, 字符串形式的JSON结构体, 如<i>"{}"</i>表示应答数据为空
    pub data: Value,
}

/// <b>异步服务应答</b>消息结构体, 用户在收到@ref AIOT_DMRECV_ASYNC_SERVICE_INVOKE 类型的异步服务调用消息后, 应发送此消息进行应答
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AsyncServiceReply {
    /// 消息标识符, uint64_t类型的整数, <b>必须与异步服务调用的消息标示符一致</b>
    pub msg_id: u64,
    /// 服务标示符, 标识了要响应服务
    pub service_id: String,
    /// 设备端状态码, 200-请求成功, 更多状态码查看<a href="https://help.aliyun.com/document_detail/89309.html">设备端通用code</a>
    pub code: u64,
    /// 设备端应答数据, 字符串形式的JSON结构体, 如<i>"{}"</i>表示应答数据为空
    pub data: Value,
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
    /// 字符串形式的JSON<b>数组</b>. 应包含用户要获取的期望属性的ID, 如<i>"[\"LightSwitch\"]"</i>
    pub params: Vec<String>,
}

/// <b>删除指定期望值</b>消息结构体
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeleteDesired {
    /// 字符串形式的JSON结构体. 应包含用户要删除的期望属性的ID和期望值版本号, 如<i>"{\"LightSwitch\":{\"version\":1},\"Color\":{}}"</i>
    pub params: Value,
}

/// <b>物模型属性上报</b>消息结构体
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PropertyBatchPost {
    /**
     * @brief 字符串形式的JSON结构体. 包含用户要批量上报的属性和事件数据, 如 {"properties":{"Power": [ { "value": "on", "time": 1524448722000 },
     *  { "value": "off", "time": 1524448722001 } ], "WF": [ { "value": 3, "time": 1524448722000 }]}, "events": {"alarmEvent": [{ "value": { "Power": "on", "WF": "2"},
     *  "time": 1524448722000}]}}
     */
    pub params: Value,
}

/// <b>物模型属性上报</b>消息结构体
/// <https://help.aliyun.com/document_detail/89301.html#title-i50-y71-kzj>
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HistoryPost {
    pub params: Vec<Value>,
}

/// data-model模块发送消息类型
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum MsgEnum {
    /// 属性上报
    PropertyPost(PropertyPost),
    /// 事件上报
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
    /// 获取期望属性值
    GetDesired(GetDesired),
    /// 清除指定的期望值
    DeleteDesired(DeleteDesired),
    /// 清除指定的期望值
    PropertyBatchPost(PropertyBatchPost),
    /// 物模型历史数据上报
    HistoryPost(HistoryPost),
}

impl DataModelMsg {
    pub fn new(data: MsgEnum) -> Self {
        Self {
            product_key: None,
            device_name: None,
            data,
        }
    }

    pub fn to_payload(&self, ack: i32) -> Result<(String, Vec<u8>)> {
        let pk = self.product_key.as_deref().unwrap_or("");
        let dn = self.device_name.as_deref().unwrap_or("");
        self.data.to_payload(pk, dn, ack)
    }
}

impl DataModelMsg {
    /// 设备上报属性
    #[inline]
    pub fn property_post(params: Value) -> Self {
        DataModelMsg::new(MsgEnum::PropertyPost(PropertyPost { params }))
    }

    /// 设备上报事件
    #[inline]
    pub fn event_post(event_id: String, params: Value) -> Self {
        DataModelMsg::new(MsgEnum::EventPost(EventPost { event_id, params }))
    }

    /// 设备设置属性响应。
    #[inline]
    pub fn property_set_reply(code: u64, data: Value, msg_id: u64) -> Self {
        DataModelMsg::new(MsgEnum::PropertySetReply(PropertySetReply {
            msg_id,
            code,
            data,
        }))
    }

    /// 设备异步服务调用响应。
    /// 当收到 `RecvEnum::AsyncServiceInvoke` 类型的数据时，调用该方法生成响应结构体。
    #[inline]
    pub fn async_service_reply(code: u64, data: Value, msg_id: u64, service_id: String) -> Self {
        DataModelMsg::new(MsgEnum::AsyncServiceReply(AsyncServiceReply {
            msg_id,
            code,
            service_id,
            data,
        }))
    }

    /// 设备同步服务调用响应。
    /// 当收到 `RecvEnum::SyncServiceInvoke` 类型的数据时，调用该方法生成响应结构体。
    /// 与异步调用不同的是，这里多了一个 `rrpc_id` 参数需要透传。
    #[inline]
    pub fn sync_service_reply(
        code: u64,
        data: Value,
        rrpc_id: String,
        msg_id: u64,
        service_id: String,
    ) -> Self {
        DataModelMsg::new(MsgEnum::SyncServiceReply(SyncServiceReply {
            rrpc_id,
            msg_id,
            code,
            service_id,
            data,
        }))
    }

    // 设备原始数据透传上报
    #[inline]
    pub fn raw_data(data: Vec<u8>) -> Self {
        DataModelMsg::new(MsgEnum::RawData(RawData { data }))
    }

    // 设备原始数据同步服务调用响应。
    #[inline]
    pub fn raw_service_reply(data: Vec<u8>, rrpc_id: String) -> Self {
        DataModelMsg::new(MsgEnum::RawServiceReply(RawServiceReply { rrpc_id, data }))
    }

    // 物模型历史数据上报
    #[inline]
    pub fn history_post(params: Vec<Value>) -> Self {
        DataModelMsg::new(MsgEnum::HistoryPost(HistoryPost { params }))
    }
}

impl MsgEnum {
    pub fn to_payload(&self, pk: &str, dn: &str, ack: i32) -> Result<(String, Vec<u8>)> {
        use MsgEnum::*;
        match &self {
            PropertyPost(data) => {
                let topic = format!("/sys/{}/{}/thing/event/property/post", pk, dn);
                let method = "thing.event.property.post";
                let payload = AlinkRequest::new(method, data.params.clone(), ack);
                Ok((topic, serde_json::to_vec(&payload)?))
            }
            EventPost(data) => {
                let topic = format!("/sys/{}/{}/thing/event/{}/post", pk, dn, data.event_id);
                let method = format!("thing.event.{}.post", data.event_id);
                let payload = AlinkRequest::new(&method, data.params.clone(), ack);
                Ok((topic, serde_json::to_vec(&payload)?))
            }
            PropertySetReply(data) => {
                let topic = format!("/sys/{}/{}/thing/service/property/set_reply", pk, dn);
                let payload = AlinkResponse::new(data.msg_id, data.code, data.data.clone());
                Ok((topic, serde_json::to_vec(&payload)?))
            }
            AsyncServiceReply(data) => {
                let topic = format!("/sys/{}/{}/thing/service/{}_reply", pk, dn, data.service_id);
                let payload = AlinkResponse::new(data.msg_id, data.code, data.data.clone());
                Ok((topic, serde_json::to_vec(&payload)?))
            }
            SyncServiceReply(data) => {
                let topic = format!(
                    "/ext/rrpc/{}/sys/{}/{}/thing/service/{}",
                    data.rrpc_id, pk, dn, data.service_id
                );
                let payload = AlinkResponse::new(data.msg_id, data.code, data.data.clone());
                Ok((topic, serde_json::to_vec(&payload)?))
            }
            RawData(data) => {
                let topic = format!("/sys/{}/{}/thing/model/up_raw", pk, dn);
                Ok((topic, data.data.clone()))
            }
            RawServiceReply(data) => {
                let topic = format!(
                    "/ext/rrpc/{}/sys/{}/{}/thing/model/down_raw_reply",
                    data.rrpc_id, pk, dn
                );
                Ok((topic, data.data.clone()))
            }
            GetDesired(data) => {
                let topic = format!("/sys/{}/{}/thing/property/desired/get", pk, dn);
                let method = "thing.property.desired.get";
                let params = Value::Array(
                    data.params
                        .iter()
                        .map(|p| Value::String(p.clone()))
                        .collect(),
                );
                let payload = AlinkRequest::new(method, params, ack);
                Ok((topic, serde_json::to_vec(&payload)?))
            }
            DeleteDesired(data) => {
                let topic = format!("/sys/{}/{}/thing/property/desired/delete", pk, dn);
                let method = "thing.property.desired.delete";
                let payload = AlinkRequest::new(method, data.params.clone(), ack);
                Ok((topic, serde_json::to_vec(&payload)?))
            }
            PropertyBatchPost(data) => {
                let topic = format!("/sys/{}/{}/thing/event/property/batch/post", pk, dn);
                let method = "thing.event.property.batch.post";
                let payload = AlinkRequest::new(method, data.params.clone(), ack);
                Ok((topic, serde_json::to_vec(&payload)?))
            }
            HistoryPost(data) => {
                let topic = format!("/sys/{}/{}/thing/event/property/history/post", pk, dn);
                let method = "thing.event.property.history.post";
                let payload = AlinkRequest::new(method, Value::Array(data.params.clone()), ack);
                Ok((topic, serde_json::to_vec(&payload)?))
            }
        }
    }
}

/// data-model模块接收消息的结构体
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DataModelRecv<T> {
    /// 消息所属设备的product_key, 不配置则默认使用MQTT模块配置的product_key
    pub product_key: String,
    /// 消息所属设备的device_name, 不配置则默认使用MQTT模块配置的device_name
    pub device_name: String,
    /// 接收消息数据
    pub data: T,
}

impl<T> DataModelRecv<T> {
    pub fn new(product_key: &str, device_name: &str, data: T) -> Self {
        Self {
            product_key: product_key.to_string(),
            device_name: device_name.to_string(),
            data,
        }
    }
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

/// <b>属性设置</b>消息结构体
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PropertySet {
    /// 消息标识符, uint64_t类型的整数
    pub msg_id: u64,
    /// 服务器下发的属性数据, 为字符串形式的JSON结构体, 此字符串<b>不</b>以结束符'\0'结尾, 如<i>"{\"LightSwitch\":0}"</i>
    pub params: Value,
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

/// <b>物模型二进制数据</b>消息结构体, 服务器的JSON格式物模型数据将通过物联网平台的JavaScript脚本转化为二进制数据, 用户在接收此消息前应确保已正确启用云端解析脚本
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RawData {
    /// 二进制数据
    pub data: Vec<u8>,
}

/// <b>二进制数据的同步服务调用</b>消息结构体, 服务器的JSON格式物模型数据将通过物联网平台的JavaScript脚本转化为二进制数据, 用户在接收此消息前应确保已正确启用云端解析脚本
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RawServiceInvoke {
    /// RRPC标识符, 用于唯一标识每一个同步服务的特殊字符串
    pub rrpc_id: String,
    /// 二进制数据
    pub data: Vec<u8>,
}
