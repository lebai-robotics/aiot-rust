use crate::alink::{AlinkRequest, AlinkResponse, ParamsRequest};
use crate::{Error, Result, ThreeTuple};
use regex::Regex;
use rumqttc::{AsyncClient, QoS};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct InitParams {
    /// 设备上传文件的名称。限制如下：
    /// - 支持数字、英文字母、下划线（_）和英文句点（.）。
    /// - 首字符仅支持数字和英文字母。
    /// - 长度不超过100字节。
    pub file_name: String,
    /// 上传文件大小，单位字节。单个文件大小不超过16 MB。
    /// 取值为-1时，表示文件大小未知。文件上传完成时，需在上传文件分片的消息中，指定参数isComplete，具体说明，请参见设备上传文件分片。
    /// 注意 若fileSize值为-1，不支持设置ficMode和ficValue。
    pub file_size: i32,
    /// 物联网平台对设备上传同名文件的处理策略。非必填参数，默认为overwrite。
    pub conflict_strategy: Option<ConflictStrategy>,
    /// 文件的完整性校验模式，目前可取值crc64。非必传参数。
    /// 若不传入，在文件上传完成后不校验文件完整性。
    /// 若传入，与ficValue同时传入，根据校验模式和校验值校验文件完整性。
    pub fic_mode: Option<FicMode>,
    /// 文件的完整性校验值，是16位的Hex格式编码的字符串。
    /// 非必传参数。若传入，与ficMode同时传入。
    pub fic_value: Option<String>,
    /// 自定义的设备请求上传文件的任务唯一ID，同一上传任务请求必须对应相同的唯一ID。
    /// 传入：用于设备请求上传文件的消息，在重发场景下的幂等性处理。
    pub init_uid: Option<String>,
    /// 设备上传文件至OSS存储空间的配置参数。非必须参数。
    /// 若未指定该参数，表示设备将文件上传至物联网平台的存储空间中。
    pub extra_params: Option<InitExtraParams>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ConflictStrategy {
    /// overwrite：覆盖模式。
    /// 先删除同名文件，再创建文件的上传任务。
    Overwrite,
    /// append：文件追加模式。
    /// 若同名文件上传未完成，设备端可根据物联网平台返回的文件信息，继续上传文件。
    /// 若同名文件上传已完成，创建文件上传任务会失败。设备端可修改文件名称或通过覆盖模式（overwrite）重新请求上传文件。
    Append,
    /// reject：拒绝模式。
    /// 物联网平台拒绝同名文件上传的请求，并返回文件已存在的错误码。
    Reject,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FicMode {
    Crc64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum OssOwnerType {
    /// iot-platform：表示上传到阿里云物联网平台的OSS存储空间中。
    /// 上传的文件会在物联网平台控制台的设备文件列表中展示，且可以通过相关的云端API管理。详细内容，请参见文件管理。
    IotPlatform,
    /// device-user：表示上传到设备所属用户自己的OSS存储空间中。
    /// 上传的文件不会在物联网平台控制台的设备文件列表中展示，也不能通过相关的云端API管理。
    /// 文件上传位置为：{ossbucket}/aliyun-iot-device-file/${instanceId}/${productKey}/${serviceId}/${deviceName}/${fileName}。
    DeviceUser,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InitExtraParams {
    /// 设备上传文件的目标OSS所有者类型。
    pub oss_owner_type: OssOwnerType,
    /// 文件上传相关的业务ID。该ID需要在物联网平台控制台预先定义。具体操作，请参见配置设备文件上传至Bucket。
    /// 仅ossOwnerType为device-user时，serviceId有效。
    pub service_id: String,
    /// 文件保存到OSS存储空间携带的标签，最多包含5个。标签定义规则，请参见对象标签。
    /// 标签Key不能以2个下划线（_）开头。
    /// 仅ossOwnerType为device-user时，fileTag有效。
    pub file_tag: Value,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct InitData {
    /// 设备上传文件的名称。
    pub file_name: String,
    /// 本次上传文件任务的标识ID。后续上传文件分片时，需要传递该文件标识ID。
    pub upload_id: String,
    /// 仅当请求参数conflictStrategy为append，且物联网平台云端存在未完成上传的文件时，返回的已上传文件的大小，单位为字节。
    pub offset: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct SendHeaderParams {
    /// 设备请求上传文件时返回的文件上传任务标识ID。
    pub upload_id: String,
    /// 已上传文件分片的总大小，单位为字节。
    pub offset: usize,
    /// 当前上传文件分片的大小，单位为字节。
    /// 非最后一个分片时，分片大小范围为256 B~131072 B。
    /// 最后一个文件分片时，若上传的文件大小已知，则分片大小范围为1 B~131072 B；若上传的文件大小未知，则分片大小范围为0 B~131072 B。
    pub b_size: usize,
    /// 仅当设备请求上传文件中fileSize为-1，即文件大小未知时，该参数有效，表示当前分片是否是文件的最后一个分片。
    /// true：是。此时，物联网平台的云端会校验已上传文件大小是否超过16 MB：
    /// 未超过：若文件大小大于0，则文件上传成功。若文件大小为0，则返回文件不能为空的错误信息且删除文件上传任务。
    /// 超过：返回文件大小超过16 MB的错误码，并删除已上传的文件。详细说明，请参见设备上传文件相关错误码。
    /// false：否。表示不是最后一个文件分片，需继续上传文件。
    pub is_complete: Option<bool>,
}

pub type SendHeader = ParamsRequest<SendHeaderParams>;

pub struct SendPayload {
    /// 表示请求Header中JSON字符串对应的字节数组长度，必须占位2个字节，高位字节在前，低位字节在后。
    /// 例如，Header的JSON字符串使用UTF-8编码转码成字节数组的长度为十进制的87，对应十六进制57，则高位字节为0x00，低位字节为0x57。
    header_length: u16,
    /// 表示请求Header中JSON字符串对应的字节数组，编码格式为UTF-8。具体内容，请参见下文的“Header的JSON数据格式”。
    header: Vec<u8>,
    /// 表示当前文件分片的字节数组，字节顺序按照相对于文件头的偏移量从小至大排列。
    file_bytes: Vec<u8>,
    /// 表示文件分片的校验值，仅支持CRC16/IBM，占位2个字节，低位字节在前，高位字节在后。
    /// 例如，文件分片的校验值为0x0809，则低位字节为0x09，高位字节为0x08。
    digest: u16,
}

/// 设备上传文件分片
/// 请求Topic：`/sys/${productKey}/${deviceName}/thing/file/upload/mqtt/send`。
impl SendPayload {
    pub fn payload(header: SendHeader, file_bytes: &[u8]) -> Result<Vec<u8>> {
        log::info!("上传文件头: {:?}", header);
        let header = serde_json::to_vec(&header)?;
        let header_length = header.len() as u16;
        let digest = super::util::crc_ibm(file_bytes);
        // let digest2 = super::util::crc_ibm2(file_bytes);
        // log::debug!("crc:[{digest:x}] [{digest2:x}] ({})", file_bytes.len());

        let mut payload = Vec::new();
        payload.extend_from_slice(&header_length.to_be_bytes());
        payload.extend_from_slice(&header);
        payload.extend_from_slice(&file_bytes);
        payload.extend_from_slice(&digest.to_le_bytes());
        Ok(payload)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct SendReplyData {
    /// 本次上传文件任务的标识ID。后续上传文件分片时，需要传递该文件标识ID。
    pub upload_id: String,
    /// 已上传文件分片的总大小，单位为字节。
    pub offset: usize,
    /// 当前上传文件分片的大小，单位为字节。
    /// 非最后一个分片时，分片大小范围为256 B~131072 B。
    /// 最后一个文件分片时，若上传的文件大小已知，则分片大小范围为1 B~131072 B；若上传的文件大小未知，则分片大小范围为0 B~131072 B。
    pub b_size: usize,
    /// 当上传了最后一个分片数据后，文件上传完成，返回该参数，值为true。
    /// 若设备请求上传文件的请求消息中fileSize值大于0，即文件大小已知时，若已上传的文件大小与设备请求上传文件时的文件大小相同，文件被识别为上传完成。
    /// 若设备请求上传的请求消息中fileSize值为-1，即文件大小未知时，若文件分片上传请求中isComplete存在且值为true，文件被识别为上传完成。
    pub complete: Option<bool>,
    /// 文件的完整性校验模式。若请求上传文件时传入了该参数，对应的值仅支持为crc64。
    pub fic_mode: Option<FicMode>,
    /// 文件上传完成，返回设备请求上传文件时的ficValue值。
    pub fic_value_client: Option<String>,
    /// 文件上传完成，返回物联网平台云端计算的文件完整性校验值。该值与ficValueClient值相同，表示文件上传完整。
    pub fic_value_server: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct UploadId {
    /// 设备请求上传文件时返回的文件上传任务标识ID。
    pub upload_id: String,
}
