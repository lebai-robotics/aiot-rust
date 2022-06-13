//! 工具类

pub mod auth;
pub mod error;

use crate::{Error, Result};
use lazy_static::lazy_static;
use rand::distributions::Alphanumeric;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::sync::Mutex;

pub const VERSION: &'static str = std::env!("CARGO_PKG_VERSION");
lazy_static! {
    pub static ref CORE_SDK_VERSION: String = format!("sdk-rust-{}", VERSION);
    static ref RNG: Mutex<StdRng> = Mutex::new(StdRng::seed_from_u64(timestamp()));
}

pub fn hex2str(input: &[u8]) -> String {
    input.iter().map(|c| format!("{:02X}", c)).collect()
}

pub fn str2hex(input: &str) -> Vec<u8> {
    fn c2u(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'A'..=b'F' => c - b'F' + 0x0A,
            b'a'..=b'f' => c - b'a' + 0x0A,
            _ => 0,
        }
    }
    let mut output = Vec::with_capacity(input.len() / 2);
    let mut iter = input.as_bytes().chunks(2);
    for chunk in iter {
        let a = c2u(chunk[0]);
        let b = c2u(chunk[1]);
        let c = (a << 4) + b;
        output.push(c);
    }
    output
}

pub fn timestamp() -> u64 {
    use std::time::SystemTime;
    if let Ok(t) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        t.as_secs()
    } else {
        0
    }
}

pub fn rand_string(len: usize) -> String {
    std::iter::repeat(())
        .map(|()| RNG.lock().unwrap().sample(Alphanumeric))
        .map(char::from)
        .take(len)
        .collect()
}

pub fn rand_u64() -> u64 {
    RNG.lock().unwrap().gen()
}

pub fn sha256(buffer: &[u8]) -> String {
    use sha2::{Digest, Sha256, Sha512};
    let mut hasher = Sha256::new();
    hasher.update(&buffer);
    let result = hasher.finalize();
    hex2str(&result)
}

pub fn md5(buffer: &[u8]) -> String {
    use md5::{Digest, Md5};
    let mut hasher = Md5::new();
    hasher.update(&buffer);
    let result = hasher.finalize();
    hex2str(&result)
}

pub fn validate(buffer: &[u8], sign_method: &str, sign: &str) -> Result<()> {
    let method = sign_method.to_ascii_lowercase();
    log::debug!("method={}, size={}", method, buffer.len());
    match method.as_str() {
        "sha256" => {
            let result = crate::util::sha256(&buffer);
            if result != sign.to_ascii_uppercase() {
                log::debug!("result:{} sign:{}", result, sign);
                return Err(Error::FileValidateFailed(method));
            }
        }
        "md5" => {
            let result = crate::util::md5(&buffer);
            if result != sign.to_ascii_uppercase() {
                log::debug!("result:{} sign:{}", result, sign);
                return Err(Error::FileValidateFailed(method));
            }
        }
        _ => {
            return Err(Error::FileValidateFailed(method));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 十六进制转字符串() {
        let v = vec![0x1, 0x2, 0x3];
        let s = hex2str(&v);
        println!("{}", s);
        let p = str2hex(&s);
        println!("{:x?}", p);
        assert_eq!(&v, &p);
    }
}
