# zos-runtime examples

ZeroOS runtime 示例：演示 `Node`、发布/订阅、定时器与 `Executor`。

## 前置条件

- 已安装 Rust toolchain
- 本机可运行 Tokio（示例使用 `#[tokio::main]`）
- Zenoh 使用默认配置（`Node::open_default()`）；若需连接外部 router，请自行调整 Zenoh 配置

## 示例列表

| 示例 | 说明 |
|------|------|
| [`pub_sub`](pub_sub.rs) | 在 `cmd_vel` 上发布/订阅 protobuf `Twist` 消息 |
| [`timer`](timer.rs) | 500ms 周期定时器，经 `Node::spin` 由 Executor 驱动 |

## 运行

在仓库根目录执行：

```bash
cargo run -p zos-runtime --example pub_sub
cargo run -p zos-runtime --example timer
```

编译所有示例（不运行）：

```bash
cargo build -p zos-runtime --examples
```

## 说明

- `pub_sub` 在同一进程内同时创建 Publisher 与 Subscriber，Subscriber 由 `node.spin()` 并发驱动。
- `timer` 仅演示定时器 + Executor；不依赖 Zenoh 话题。
- MuJoCo / Unitree Go2 仿真见 [`../simulation/README.md`](../simulation/README.md)（Python，与本目录 Rust 示例分开）。
