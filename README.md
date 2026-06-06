# ZeroOS

个人练习项目：用 Rust 摸索机器人软件栈（通信、建图、定位、导航、感知、Agent 等），**非生产可用**，接口与结构可能随时调整。

**当前有实质代码**的主要是通信运行时（[`zos-runtime`](crates/runtime/)）；其余目录多为占位或随手试验。

## 练习方向（非承诺路线图）

| 方向 | 状态 | 说明 |
|------|------|------|
| **Runtime**（中间件） | 在练 | 进程间通信与节点模型，见 [`crates/runtime`](crates/runtime/) |
| **建图（Mapping）** | 占位 | [`zos-map`](crates/map/) |
| **定位（Localization）** | 未开始 | 位姿估计与坐标系 |
| **导航（Navigation）** | 未开始 | 路径规划与运动控制 |
| **感知（Perception）** | 未开始 | 传感器与场景理解 |
| **Agent** | 未开始 | 任务编排与决策 |
| **仿真** | 在练 | [`zos-simulation`](crates/simulation/)（MuJoCo 差分轮） |

## 仓库结构

| Crate | 目录 | 说明 |
|-------|------|------|
| `zos-runtime` | [`crates/runtime`](crates/runtime/) | 通信中间件 → [README](crates/runtime/README.md) |
| `zos-msg` | [`crates/msg`](crates/msg/) | 跨模块消息定义 |
| `zos-map` | [`crates/map`](crates/map/) | 地图（占位） |
| `zos` | [`crates/zos`](crates/zos/) | 门面：`runtime` / `msg` / `map` |

使用 runtime（构建、示例、API、依赖）请阅读 **[`crates/runtime/README.md`](crates/runtime/README.md)**。

## 环境要求

- Rust toolchain（edition 2024）
- **protoc**（编译 `zos-msg` 的 `.proto`）：macOS `brew install protobuf`，Debian/Ubuntu `sudo apt-get install protobuf-compiler`
- 仿真：MuJoCo + `mujoco-rs`，见 [`crates/simulation/README.md`](crates/simulation/README.md)

## 构建

在仓库根目录：

```bash
cargo build
```

## 依赖（应用）

```toml
zos = { path = "crates/zos" }
```

## 许可证

Apache-2.0；源码文件含 SPDX 头。仿真资源许可见 [`crates/simulation/README.md`](crates/simulation/README.md)。
