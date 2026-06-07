# ZeroOS

**试验性探索项目**：用 Rust 探索机器人软件栈（通信、建图、定位、导航、感知、Agent 等）。**非生产可用**；接口、目录与实现可能随时调整，不保证向后兼容。

## 当前现状

仓库里**已有可运行代码**的主要有三块，其余多为占位或随手试验：

| 模块 | Crate | 能做什么 |
|------|-------|----------|
| **通信运行时** | [`zos-runtime`](crates/runtime/) | ROS 2 风格 Node、pub/sub、Service/Client、Executor；底层 [Zenoh](https://zenoh.io/)，上层 `mw` trait 抽象 |
| **仿真** | [`zos-simulation`](crates/simulation/) | MuJoCo 差分轮小车 + 办公场景，键盘遥控，里程计/IMU 读取（尚未接 Runtime） |
| **可视化** | [`zos-viz`](crates/viz/) | 桌面 egui，经 Runtime/Zenoh 订阅 `cmd_vel` 等话题 |
| **消息** | [`zos-msg`](crates/msg/) | Protobuf 定义（当前主要是 `Twist` / `Vector2`） |
| **地图** | [`zos-map`](crates/map/) | 占位 |
| **门面** | [`zos`](crates/zos/) | 聚合 `runtime` / `msg` / `map` |

典型联调路径：`zos-runtime` 示例发布 `cmd_vel` → `zos-viz` 订阅显示。仿真与 Runtime 的打通仍在规划中。

## 探索方向

以下为**可能涉及**的方向，**不是路线图、无时间表**；做到哪算哪，随时可能搁置或推翻。

| 方向 | 进展 | 想摸索的内容 |
|------|------|----------------|
| **通信 / Runtime** | 雏形 | ROS 2 风格节点与中间件抽象（Zenoh + `mw` trait）→ [README](crates/runtime/README.md) |
| **仿真** | 雏形 | MuJoCo 差分轮、办公场景、遥控与真值传感器 → [README](crates/simulation/README.md) |
| **可视化** | 雏形 | Runtime 话题监控（egui）→ [README](crates/viz/README.md) |
| **联调** | 未开始 | 仿真 ↔ Runtime：`cmd_vel` 驱动、`odom` / `scan` 发布 |
| **消息** | 少量 | 扩展 `LaserScan`、`CompressedImage` 等 Protobuf |
| **建图** | 占位 | 栅格地图、图层 → [`zos-map`](crates/map/) |
| **定位** | 未开始 | 位姿估计、坐标系与 TF 类抽象 |
| **导航** | 未开始 | 路径规划、局部避障、跟踪 `cmd_vel` |
| **感知** | 未开始 | 相机 / 激光等传感器管线 |
| **Agent** | 未开始 | 任务编排与高层决策 |

## 仓库结构

```
crates/
├── runtime/      # zos-runtime — Node / Executor + mw 中间件层
├── msg/          # zos-msg — Protobuf 消息
├── map/          # zos-map — 地图（占位）
├── zos/          # 门面 crate
├── simulation/   # zos-simulation — MuJoCo 仿真
└── viz/          # zos-viz — 消息可视化
```

Runtime 的分层设计、API 与示例见 **[`crates/runtime/README.md`](crates/runtime/README.md)**。

## 快速尝试

**Runtime pub/sub + 可视化**（两个终端）：

```bash
# 终端 1：发布 cmd_vel
cargo run -p zos-runtime --example pub_sub

# 终端 2：订阅并显示
cargo run -p zos-viz
```

**仿真**（需安装 MuJoCo，见 [simulation README](crates/simulation/README.md)）：

```bash
# macOS 示例
brew install kcking/tap/mujoco@3.7
export PKG_CONFIG_PATH="$(brew --prefix mujoco@3.7)/lib/pkgconfig"
cargo run -p zos-simulation
```

键盘 `W`/`S`/`A`/`D` 遥控，窗口内显示当前 `cmd_vel`。

## 环境要求

| 依赖 | 用途 | 安装 |
|------|------|------|
| Rust（edition 2024） | 全仓库 | [rustup](https://rustup.rs/) |
| **protoc** | 编译 `zos-msg` | macOS `brew install protobuf`；Debian/Ubuntu `sudo apt-get install protobuf-compiler` |
| **MuJoCo 3.3.7** | 仅 `zos-simulation` viewer | 见 [simulation README](crates/simulation/README.md) |
| 图形环境 | `zos-simulation` / `zos-viz` 完整构建 | 本地桌面；CI 仅无头编译 |

## 构建与测试

```bash
# 默认构建（simulation / viz 会拉 MuJoCo / GUI 依赖）
cargo build

# 与 CI 一致的无头测试（无需 MuJoCo、无需显示）
cargo test --workspace --exclude zos-simulation --exclude zos-viz
cargo test -p zos-simulation --no-default-features
cargo test -p zos-viz --no-default-features
cargo build -p zos-runtime --examples
```

更多 Runtime 示例：`cargo run -p zos-runtime --example service` 等，见 [`crates/runtime/examples/README.md`](crates/runtime/examples/README.md)。

## 依赖（应用）

```toml
zos = { path = "crates/zos" }
# 或按需单独依赖
zos-runtime = { path = "crates/runtime" }
zos-msg = { path = "crates/msg" }
```

## 许可证

Apache-2.0；源码文件含 SPDX 头。仿真资源许可见 [`crates/simulation/README.md`](crates/simulation/README.md)。
