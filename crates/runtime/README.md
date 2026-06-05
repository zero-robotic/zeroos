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
use zos_runtime::{init, InitOptions, RuntimeError};

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    init(InitOptions::new()).await?;
    // 或 init(InitOptions::new().config(my_zenoh_config)).await?;
    Ok(())
}
```

## 快速开始

```rust
use zos_msg::Twist;
use zos_runtime::{init, InitOptions, Node, NodeOptions, RuntimeError};

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    init(InitOptions::new()).await?;
    let mut node = Node::new(NodeOptions::new()).await?;

    node.create_subscriber_builder::<Twist>("cmd_vel")
        .register(|msg| async move {
            println!("linear = {}", msg.linear);
        });

    let publisher = node.create_publisher::<Twist>("cmd_vel").build().await?;
    publisher.publish(&Twist { linear: 1.0, angular: 0.0 }).await?;

    node.spin().await
}
```

带 **namespace** 的节点（与 ROS 2 `__ns` 一致）：

```rust
init(InitOptions::new()).await?;
let mut node = Node::new(
    NodeOptions::new()
        .name("server")
        .namespace("/demo"),
)
.await?;

node.create_service_builder::<Req, Resp>("scale")
    .register(|req| async move { Ok(resp) });
```

## 核心类型

| 类型 | ROS 2 对应 | 说明 |
|------|------------|------|
| [`init`](src/context.rs) | `rclcpp::init` | 全局 Zenoh session，每进程一次 |
| [`InitOptions`](src/context.rs) | init 选项 | Zenoh `config` |
| [`Node`](src/node.rs) | `rclcpp::Node` | 创建端点、`spin()`（复用全局 session） |
| [`NodeOptions`](src/node.rs) | node 选项 | `name`、`namespace`（默认 `/`）、`executor` |
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

在创建节点时通过 [`ExecutorOptions`](src/executor.rs) 配置，统一用 `node.spin().await`：

```rust
init(InitOptions::new()).await?;
// 使用 #[tokio::main] 的线程池（默认）
Node::new(NodeOptions::new()).await?;

// 专用 n 线程池
Node::new(
    NodeOptions::new().executor(ExecutorOptions::new().worker_threads(2)),
)
.await?;
```

| `executor.worker_threads` | 行为 |
|---------------------------|------|
| `None` | 当前 Tokio runtime（由 `#[tokio::main(worker_threads = N)]` 决定） |
| `Some(n)` | 独立 `n` worker 线程池 |

手动组装时：`Executor::new(opts).add_node(&mut node).spin().await`（默认选项可用 `Executor::default()`）。

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
