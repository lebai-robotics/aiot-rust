//! 签名和证书

use crate::{Error, Result};

pub const SIGN_METHOD: &str = "hmacsha256";

pub fn sign(input: &str, key: &str) -> String {
    use super::hex2str;
    use hmac::{Hmac, Mac, NewMac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    match HmacSha256::new_from_slice(key.as_bytes()) {
        Ok(mut mac) => {
            mac.update(input.as_bytes());
            let result = mac.finalize();
            let result = result.into_bytes().to_vec();
            hex2str(&result)
        }
        Err(_) => input.to_string(),
    }
}

pub fn sign_device(uuid: &str, dn: &str, pk: &str, ds: &str, timestamp: u128) -> String {
    let res = format!(
        "clientId{}deviceName{}productKey{}timestamp{}",
        uuid, dn, pk, timestamp
    );
    sign(&res, ds)
}

pub mod mqtt {
    const CORE_AUTH_TIMESTAMP: u128 = 2524608000000;

    pub fn username(product_key: &str, device_name: &str) -> String {
        format!("{}&{}", device_name, product_key)
    }

    pub fn password(
        product_key: &str,
        device_name: &str,
        device_secret: &str,
        assigned_client_id: bool,
    ) -> String {
        let uuid = if assigned_client_id {
            "".to_string()
        } else {
            format!("{}.{}", product_key, device_name)
        };
        super::sign_device(
            &uuid,
            device_name,
            product_key,
            device_secret,
            CORE_AUTH_TIMESTAMP,
        )
    }

    pub fn client_id(
        product_key: &str,
        device_name: &str,
        secure_mode: &str,
        extend_client_id: &str,
        assigned_client_id: bool,
    ) -> String {
        let dest = format!(
            "|timestamp={},_ss=1,_v={},securemode={},signmethod={},ext=3,{}|",
            CORE_AUTH_TIMESTAMP,
            *crate::util::CORE_SDK_VERSION,
            secure_mode,
            super::SIGN_METHOD,
            extend_client_id
        ); // ext bitmap: bit0-rrpc, bit1-ext_notify
        if assigned_client_id {
            dest
        } else {
            format!("{}.{}{}", product_key, device_name, dest)
        }
    }
}

pub mod http {
    pub fn client_id(product_key: &str, device_name: &str) -> String {
        format!("{}.{}", product_key, device_name)
    }

    pub fn password(product_key: &str, device_name: &str, device_secret: &str) -> String {
        let text = format!(
            "clientId{}deviceName{}productKey{}",
            client_id(product_key, device_name),
            device_name,
            product_key
        );
        super::sign(&text, device_secret)
    }
}

/// 阿里云物联网平台的 X509 根证书
pub const ALI_CA_CERT: &str = r#"-----BEGIN CERTIFICATE-----
MIIDdTCCAl2gAwIBAgILBAAAAAABFUtaw5QwDQYJKoZIhvcNAQEFBQAwVzELMAkG
A1UEBhMCQkUxGTAXBgNVBAoTEEdsb2JhbFNpZ24gbnYtc2ExEDAOBgNVBAsTB1Jv
b3QgQ0ExGzAZBgNVBAMTEkdsb2JhbFNpZ24gUm9vdCBDQTAeFw05ODA5MDExMjAw
MDBaFw0yODAxMjgxMjAwMDBaMFcxCzAJBgNVBAYTAkJFMRkwFwYDVQQKExBHbG9i
YWxTaWduIG52LXNhMRAwDgYDVQQLEwdSb290IENBMRswGQYDVQQDExJHbG9iYWxT
aWduIFJvb3QgQ0EwggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQDaDuaZ
jc6j40+Kfvvxi4Mla+pIH/EqsLmVEQS98GPR4mdmzxzdzxtIK+6NiY6arymAZavp
xy0Sy6scTHAHoT0KMM0VjU/43dSMUBUc71DuxC73/OlS8pF94G3VNTCOXkNz8kHp
1Wrjsok6Vjk4bwY8iGlbKk3Fp1S4bInMm/k8yuX9ifUSPJJ4ltbcdG6TRGHRjcdG
snUOhugZitVtbNV4FpWi6cgKOOvyJBNPc1STE4U6G7weNLWLBYy5d4ux2x8gkasJ
U26Qzns3dLlwR5EiUWMWea6xrkEmCMgZK9FGqkjWZCrXgzT/LCrBbBlDSgeF59N8
9iFo7+ryUp9/k5DPAgMBAAGjQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8E
BTADAQH/MB0GA1UdDgQWBBRge2YaRQ2XyolQL30EzTSo//z9SzANBgkqhkiG9w0B
AQUFAAOCAQEA1nPnfE920I2/7LqivjTFKDK1fPxsnCwrvQmeU79rXqoRSLblCKOz
yj1hTdNGCbM+w6DjY1Ub8rrvrTnhQ7k4o+YviiY776BQVvnGCv04zcQLcFGUl5gE
38NflNUVyRRBnMRddWQVDf9VMOyGj/8N7yy5Y0b2qvzfvGn9LhJIZJrglfCm7ymP
AbEVtQwdpf5pLGkkeB6zpxxxYu7KyJesF12KwvhHhm4qxFYxldBniYUr+WymXUad
DKqC5JlR3XC321Y9YeRq4VzW9v493kHMB65jUr9TU/Qr6cf9tveCX4XSQRjbgbME
HMUfpIBvFSDJ3gyICh3WZlXi/EjJKSZp4A==
-----END CERTIFICATE-----"#;

fn aliyun_root_cert_store() -> Result<rustls::RootCertStore> {
    let mut store = rustls::RootCertStore::empty();
    let mut cred = ALI_CA_CERT.clone().as_bytes();
    let mut items = rustls_pemfile::certs(&mut cred).map_err(|err| {
        log::error!("rustls_pemfile::certs {err}");
        Error::AddPemFileError
    })?;
    for item in items {
        let cert = rustls::Certificate(item);
        store.add(&cert).map_err(|err| {
            log::error!("RootCertStore.add {err}");
            Error::AddPemFileError
        })?;
    }
    Ok(store)
}

pub fn aliyun_client_config() -> Result<rustls::ClientConfig> {
    let builder = rustls::ClientConfig::builder();
    let config = builder
        .with_safe_defaults()
        .with_root_certificates(aliyun_root_cert_store()?)
        .with_no_client_auth();
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 测试连接签名() {
        let product_key = "a13FN5TplKq";
        let device_name = "mqtt_basic_demo";
        let device_secret = "jA0K15GobTDa5wgOtJPzdtcZPc4X7NYQ";
        let output = "4780A5F17990D8DC4CCAD392683ED80160C4C2A1FFA649425CD0E2666A8593EB";
        assert_eq!(
            &output,
            &mqtt::password(product_key, device_name, device_secret, false)
        );
    }
}
