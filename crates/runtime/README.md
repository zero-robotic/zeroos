# zos-runtime

基于 [Zenoh](https://zenoh.io/) 的 ZeroOS 运行时：提供与 ROS 2 相近的 **Node**、发布/订阅、**Service** / **Client**、定时器与执行器。消息类型来自 [`zos-msg`](../msg/)（Protobuf）。

## 依赖

在 workspace 或其它 crate 的 `Cargo.toml` 中：

```toml
zos-runtime = { path = "../runtime" }  # 或 workspace 依赖名
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

应用代码需要 Tokio 运行时（`#[tokio::main]`）。

## 初始化

进程内先调用一次 [`init`](src/context.rs) 打开全局 Zenoh session，再创建任意个 [`Node`](src/node.rs)（共享同一 session，类似 ROS 2 `rclcpp::init`）：

```rust
use zos_runtime::{init, init_from_file, RuntimeError};

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    init().await?;
    // 或 init_from_file("zenoh.json5").await?;
    Ok(())
}
```

## 快速开始

```rust
use zos_msg::Twist;
use zos_runtime::{init, Executor, Node, NodeOptions, RuntimeError};

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    init().await?;
    let mut node = Node::new(NodeOptions::new());

    node.create_subscriber_builder::<Twist>("cmd_vel")
        .register(|msg| async move {
            println!("linear = {}", msg.linear);
        })?;

    let publisher = node.create_publisher::<Twist>("cmd_vel").build().await?;
    publisher.publish(&Twist { linear: 1.0, angular: 0.0 }).await?;

    Executor::spin_node(&mut node).await
}
```

带 **namespace** 的节点（与 ROS 2 `__ns` 一致）：

```rust
init().await?;
let mut node = Node::new(
    NodeOptions::new()
        .name("server")
        .namespace("/demo"),
);

node.create_service_builder::<Req, Resp>("scale")
    .register(|req| async move { Ok(resp) })?;
```

## 核心类型

| 类型 | ROS 2 对应 | 说明 |
|------|------------|------|
| [`init`](src/context.rs) | `rclcpp::init` | 默认配置，每进程一次 |
| [`init_from_file`](src/context.rs) | 带配置 init | JSON5 配置文件路径 |
| [`session`](src/context.rs) | — | 全局 session（[`init`](src/context.rs) 后按需 clone） |
| [`Node`](src/node.rs) | `rclcpp::Node` | 创建端点、收集 runnable |
| [`NodeOptions`](src/node.rs) | node 选项 | `name`、`namespace`（默认 `/`） |
| [`Publisher`](src/publisher.rs) | `Publisher` | 话题发布 |
| [`Subscriber`](src/subscriber.rs) | `Subscription` | 话题订阅，可注册进 executor |
| [`Service`](src/service.rs) | `Service` | 请求/响应服务端 |
| [`Client`](src/client.rs) | `Client` | 请求/响应客户端 |
| [`Timer`](src/timer.rs) | `Timer` | 周期或单次定时 |
| [`Executor`](src/executor.rs) | executor | 并发驱动已注册的 `Runnable` |

各类型配有 **Builder**（如 `PublisherBuilder`），由 `Node::create_*` 返回；实现位于对应源文件。

## 命名（namespace）

与 ROS 2 相同，由 [`resolve_name`](src/node.rs) 解析：

- **相对名** `scale` → `{namespace}/scale`（根 namespace 下即为 `scale`）
- **绝对名** `/demo/scale` → 始终为 `demo/scale`，**忽略**当前节点的 namespace
- **节点名** `NodeOptions::name` 仅作标识，**不会**拼进话题或服务路径

跨 namespace 调用服务时使用绝对名，例如 `create_client("/demo/scale")`。

## Executor 与线程池

注册 subscriber / timer / service 后，由 [`Executor`](src/executor.rs) 驱动：

```rust
use zos_runtime::{Executor, ExecutorOptions};

init().await?;
let mut node = Node::new(NodeOptions::new());
// ... register runnables on node ...

// 使用 #[tokio::main] 的线程池（默认）
Executor::spin_node(&mut node).await?;

// 专用 n 线程池
Executor::spin_node_with(&mut node, ExecutorOptions::new().worker_threads(2)).await?;
```

| `ExecutorOptions::worker_threads` | 行为 |
|-----------------------------------|------|
| `None` | 当前 Tokio runtime（由 `#[tokio::main(worker_threads = N)]` 决定） |
| `Some(n)` | 独立 `n` worker 线程池 |

多节点时手动组装：`Executor::new(opts)` → `add_node`（可多次）→ `spin().await`。

## 示例

详见 [`examples/README.md`](examples/README.md)。

```bash
cargo run -p zos-runtime --example pub_sub
cargo run -p zos-runtime --example executor
cargo run -p zos-runtime --example service
cargo run -p zos-runtime --example client
```

## 测试

```bash
cargo test -p zos-runtime
```

当前单元测试覆盖 namespace / 名称解析逻辑（`node` 模块）。

## 规划

`Parameter`、`Logger`、`Lifecycle` 等见 [`src/lib.rs`](src/lib.rs) 顶部说明。
