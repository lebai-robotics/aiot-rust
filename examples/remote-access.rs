use aiot::{
    LocalService, MqttClient, RemoteAccessRecv, SecureTunnelNotify, ThreeTuple, TunnelParams,
    TunnelProxy,
};
use anyhow::Result;
use log::*;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let host = "iot-as-mqtt.cn-shanghai.aliyuncs.com";
    let three = ThreeTuple::from_env();
    let mut conn = MqttClient::new_public_tls(host, &three)?.connect();

    // proxy 可以是完全独立的进程
    let proxy = TunnelProxy::new();
    let ssh = LocalService::default(); // 默认是 _SSH 127.0.0.1:22
    proxy.add_service(ssh).await?;

    let mut ra = conn.remote_access()?;
    ra.init().await?;
    loop {
        tokio::select! {
            Ok(_) = conn.poll() => {
                // 主循环的 poll 是必须的
            }
            Ok(data) = ra.poll() => {
                let notify = match data {
                    RemoteAccessRecv::Switch(data) => data,
                    RemoteAccessRecv::RequestReply(data) => data.data,
                };
                match notify {
                    SecureTunnelNotify::Connect(data) => {
                        info!("Connect = {:?}", data);
                        let params = TunnelParams {
                            id: data.tunnel_id,
                            host: data.host,
                            port: format!("{}", data.port),
                            path: data.path,
                            token: data.token,
                        };
                        proxy.add_tunnel(params).await.ok();
                    }
                    SecureTunnelNotify::Close(data) => {
                        info!("Close = {:?}", data);
                    }
                }
            }
        }
    }
}
