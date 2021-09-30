[![Crates.io](https://img.shields.io/crates/v/aiot.svg)](https://crates.io/crates/aiot)
[![Workflow Status](https://github.com/lebai-robotics/aiot-rust/actions/workflows/rust.yml/badge.svg)](https://github.com/lebai-robotics/aiot-rust/actions/workflows/rust.yml)

# Rust Link SDK (aiot-rs)

提供阿里云物联网平台的设备端 Rust 开发工具包（非阿里官方）。

阿里官方的 [Link SDK](https://help.aliyun.com/document_detail/96596.html) 提供了以下语言的版本：

- [C Link SDK](https://help.aliyun.com/document_detail/163753.html)
- [Android Link SDK](https://help.aliyun.com/document_detail/96605.html)
- [Node.js Link SDK](https://help.aliyun.com/document_detail/96617.html)
- [Java Link SDK](https://help.aliyun.com/document_detail/97330.html)
- [Python Link SDK](https://help.aliyun.com/document_detail/98291.html)
- [iOS Link SDK](https://help.aliyun.com/document_detail/100532.html)

其中，C Link SDK 是功能最完整的，我们的 Rust SDK 也是对标这个进行设计开发的。 刚开始使用 Rust 时候，尝试基于 C SDK 在 FFI 基础上进行封装，发现这种方式性能和可扩展性都不高，于是基于 `rumqttc`
和 `tokio` 实现了现在的版本。

本项目遵循阿里云物联网平台定义的 [Alink 协议](https://help.aliyun.com/document_detail/90459.html)，实现并正在实现如下功能：

- [ ] 设备认证与接入
    - [x] MQTT接入
    - [ ] CoAP接入
    - [x] HTTPS接入
    - [ ] X.509证书接入
- [x] 消息通信
    - [x] RRPC
    - [x] 广播通信
- [ ] 设备管理
    - [x] 物模型
    - [ ] 数字孪生 (NEW)
    - [x] 设备标签
    - [x] 设备影子
    - [x] 子设备管理
    - [ ] 文件管理
    - [ ] 设备签名
    - [ ] 设备任务
    - [x] 时间同步
    - [ ] 设备分发
- [ ] 监控运维
    - [x] 日志服务
    - [ ] 设备诊断
    - [x] 远程登录
    - [x] 设备OTA
    - [x] 远程配置

本项目仍在开发中，如有问题请提出 Issue 或者直接提交 Pull Request。目前没有移植 `no_std` 的计划。

### 示例代码

```bash
source demo.env # 初始化三元组环境变量，仅用于演示
cargo run --example mqtt-basic # MQTT 基础示例
cargo run --example mqtt-rrpc # MQTT RRPC 通信示例
cargo run --example mqtt-broadcast # MQTT 广播通信示例
cargo run --example data-model-basic # 物模型基础示例
cargo run --example dynregmq-basic # 设备“一型一密”动态注册示例
cargo run --example remote-access # 设备远程登录示例
cargo run --example http-basic # HTTP 连接示例
```
