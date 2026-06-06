# zos-runtime examples

ZeroOS runtime 示例：演示 `Node`、发布/订阅、定时器、Service 与 Client。

## 前置条件

- 已安装 Rust toolchain
- 本机可运行 Tokio（示例使用 `#[tokio::main]`）
- 先 `init().await?`（自定义配置用 `init_from_file("zenoh.json5")`），再 `Node::new(...)`

## 示例列表

| 示例 | 说明 |
|------|------|
| [`pub_sub`](pub_sub.rs) | 在 `cmd_vel` 上发布/订阅 protobuf `Twist` 消息 |
| [`timer`](timer.rs) | 500ms 周期定时器，经 `Executor::spin` 驱动 |
| [`executor`](executor.rs) | `ExecutorOptions` 默认池 vs 专用线程池 |
| [`service`](service.rs) | [`Service`] 服务端：`Twist` → `Vector2`，等待请求 |
| [`client`](client.rs) | [`Client`] 调用 `demo/scale`；默认自带进程内服务端 |

## 运行

在仓库根目录执行：

```bash
cargo run -p zos-runtime --example pub_sub
cargo run -p zos-runtime --example timer
cargo run -p zos-runtime --example executor
cargo run -p zos-runtime --example executor -- --dedicated
cargo run -p zos-runtime --example service
cargo run -p zos-runtime --example client
```

Service + Client 分进程演示（两个终端）：

```bash
# 终端 1
cargo run -p zos-runtime --example service

# 终端 2
cargo run -p zos-runtime --example client -- --remote
```

编译所有示例（不运行）：

```bash
cargo build -p zos-runtime --examples
```

## 说明

- `pub_sub` 在同一进程内同时创建 Publisher 与 Subscriber，Subscriber 由 `Executor::spin()` 并发驱动。
- `timer` 仅演示定时器 + Executor；不依赖 Zenoh 话题。
- `executor` 演示 `ExecutorOptions` 的两种线程模型；加 `--dedicated` 切换专用线程池。
- `service` 在 namespace `/demo` 上注册 `scale`（全名 `/demo/scale`）；需另开终端运行 `client -- --remote`。
- `client` 用绝对名 `create_client("/demo/scale")` 调用（与 ROS 2 一致，不受 caller 的 namespace 影响）；默认模式在进程内启动同 namespace 的服务端。
- MuJoCo 差分轮仿真见 [`../simulation/README.md`](../simulation/README.md)（`cargo run -p zos-simulation`）。
