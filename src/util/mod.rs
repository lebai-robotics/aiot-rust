pub mod auth;
pub mod error;

use lazy_static::lazy_static;
use rand::distributions::Alphanumeric;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::sync::Mutex;

pub const VERSION: Option<&'static str> = std::option_env!("CARGO_PKG_VERSION");
lazy_static! {
    pub static ref CORE_SDK_VERSION: String = format!("sdk-rust-{}", VERSION.unwrap_or("unknown"));
    pub static ref RNG: Mutex<StdRng> = Mutex::new(StdRng::seed_from_u64(timestamp()));
}

pub fn hex2str(input: &[u8]) -> String {
    input.iter().map(|c| format!("{:02X}", c)).collect()
}

pub fn str2hex(input: &str) -> Vec<u8> {
    fn c2u(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'A'..=b'F' => c - b'F',
            b'a'..=b'f' => c - b'a',
            _ => 0,
        }
    }
    let mut output = Vec::with_capacity(input.len() / 2);
    let mut iter = input.as_bytes().chunks(2);
    while let Some(chunk) = iter.next() {
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
