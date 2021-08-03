use aiot::{Http, ThreeTuple};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let host = "iot-as-http.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple::from_env();

    let mut client = Http::new_tls(&host, &three)?;
    client.auth().await?;

    let topic = "/sys/a13FN5TplKq/http_basic_demo/thing/event/property/post";
    let data = b"{\"id\":\"1\",\"version\":\"1.0\",\"params\":{\"LightSwitch\":1}}";
    let res = client.send(topic, data).await?;
    log::info!("{:?}", res);

    Ok(())
}
