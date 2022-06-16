pub mod protocol;
pub mod proxy;
pub mod session;

/// tunnel内部事件类型
pub enum Event {
    /// 当tunnel实例连接代理通道成功
    Connect,
    /// 当tunnel实例从代理通道断开
    Disconnect,
    /// 隧道认证信息已经过期，需要重新连接
    Expired,
}
