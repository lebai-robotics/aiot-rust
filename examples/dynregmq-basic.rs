use aiot::DynamicRegister;
use anyhow::Result;
use log::*;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let product_key = std::env::var("PRODUCT_KEY")?;
    let device_name = std::env::var("DEVICE_NAME")?;
    let product_secret = std::env::var("PRODUCT_SECRET")?;

    let reg = DynamicRegister::new_tls(&host, &product_key, &product_secret, &device_name)?;
    let res = reg.register().await?;
    info!("{:?}", res);

    Ok(())
}
