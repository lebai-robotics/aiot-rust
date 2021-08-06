use crate::util::auth;
use crate::ThreeTuple;
use log::*;
use reqwest::{Certificate, ClientBuilder};
use serde::{Deserialize, Serialize};

type Result<T> = core::result::Result<T, HttpError>;

#[derive(Debug)]
pub struct Http {
    pub host: String,
    pub three: ThreeTuple,
    pub token: Option<String>,
    client: reqwest::Client,
    pub extend: Option<String>,
}

impl Http {
    pub fn new_tls(host: &str, three: &ThreeTuple) -> crate::Result<Self> {
        let cred = Certificate::from_pem(auth::ALI_CA_CERT.as_bytes())?;
        let client = ClientBuilder::new()
            .add_root_certificate(cred)
            .http1_title_case_headers()
            .build()?;
        Ok(Self {
            host: host.to_string(),
            three: three.clone(),
            token: None,
            client,
            extend: None,
        })
    }

    fn extend_devinfo(&self) -> &str {
        if let Some(s) = &self.extend {
            s
        } else {
            ""
        }
    }

    pub async fn auth(&mut self) -> crate::Result<()> {
        let url = format!(
            "https://{}/auth?_v={}&{}",
            self.host,
            *crate::util::CORE_SDK_VERSION,
            self.extend_devinfo()
        );
        let body = HttpAuthBody {
            product_key: self.three.product_key.to_string(),
            device_name: self.three.device_name.to_string(),
            client_id: auth::http::client_id(&self.three.product_key, &self.three.device_name),
            // timestamp: None,
            sign: auth::http::password(
                &self.three.product_key,
                &self.three.device_name,
                &self.three.device_secret,
            ),
            signmethod: Some(auth::SIGN_METHOD.to_string()),
        };
        debug!("{}", serde_json::to_string(&body)?);
        let res = self.client.post(&url).json(&body).send().await?;
        debug!("{:?}", res);
        let res: HttpResponse = res.json().await?;
        debug!("{:?}", res);
        // let res: InfoToken = res.try_into()?;
        // debug!("{:?}", res);
        self.token = Some(res.get(true, "token")?);
        Ok(())
    }

    // 返回 messageId
    pub async fn send(&mut self, topic: &str, data: &[u8]) -> crate::Result<String> {
        let token = self.token.clone().ok_or(HttpError::TokenIsNull)?;
        let url = format!("https://{}/topic{}", self.host, topic);
        let res = self
            .client
            .post(url)
            .header("Content-Type", "application/octet-stream")
            .header("password", token)
            .body(data.to_vec());
        debug!("{:?}", res);
        let res = res.send().await?;
        debug!("{:?}", res);
        let res: HttpResponse = res.json().await?;
        debug!("{:?}", res);
        Ok(res.get(false, "messageId")?)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HttpAuthBody {
    #[serde(rename = "productKey")]
    pub product_key: String,
    #[serde(rename = "deviceName")]
    pub device_name: String,
    #[serde(rename = "clientId")]
    pub client_id: String,
    // pub timestamp: Option<String>,
    pub sign: String,
    pub signmethod: Option<String>,
    // pub version: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HttpResponse {
    pub code: i32,
    pub message: String,
    pub info: Option<serde_json::Value>,
}

impl HttpResponse {
    pub fn get(&self, is_auth: bool, key: &str) -> Result<String> {
        use HttpError::*;
        match self.code {
            0 => match &self.info {
                Some(data) => match data.get(key) {
                    Some(data) => match data.as_str() {
                        Some(data) => Ok(data.to_string()),
                        None => Err(ParseError),
                    },
                    None => Err(ParseError),
                },
                None => Err(ParseError),
            },
            10000 => Err(CommonError),
            10001 => Err(ParamError),
            20000 => Err(AuthCheckError),
            20001 => Err(TokenIsExpired),
            20002 => {
                // https://help.aliyun.com/document_detail/58034.html
                // 阿里云平台两个接口返回的错误码定义不一样
                if is_auth {
                    Err(UpdateSessionError)
                } else {
                    Err(TokenIsNull)
                }
            }
            20003 => Err(CheckTokenError),
            30001 => Err(PublishMessageError),
            40000 => Err(RequestTooMany),
            _ => Err(Unknown(self.code, self.message.clone())),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum HttpError {
    #[error("未知错误")]
    CommonError,
    #[error("请求的参数异常")]
    ParamError,
    #[error("设备鉴权失败")]
    AuthCheckError,
    #[error("更新失败")]
    UpdateSessionError,
    #[error("解析失败")]
    ParseError,
    #[error("请求次数过多，流控限制")]
    RequestTooMany,
    #[error("token失效。需重新调用auth进行鉴权，获取token")]
    TokenIsExpired,
    #[error("请求header中无token信息")]
    TokenIsNull,
    #[error("根据token获取identify信息失败。需重新调用auth进行鉴权，获取token")]
    CheckTokenError,
    #[error("数据上行失败")]
    PublishMessageError,
    #[error("未知错误 {1} 错误码 {0}")]
    Unknown(i32, String),
}
