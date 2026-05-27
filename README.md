# ZeroOS

面向机器人的操作系统与软件栈（Rust），覆盖通信、建图、定位、导航、感知与智能体等能力。

**当前已实现**的主要是通信运行时（[`zos-runtime`](crates/runtime/)）；其余模块在规划中或仅占位。

## 规划模块

| 模块 | 状态 | 说明 |
|------|------|------|
| **Runtime**（中间件） | 已实现 | 进程间通信与节点模型，见 [`crates/runtime`](crates/runtime/) |
| **建图（Mapping）** | 规划中 | [`zos-map`](crates/map/) |
| **定位（Localization）** | 规划中 | 位姿估计与坐标系 |
| **导航（Navigation）** | 规划中 | 路径规划与运动控制 |
| **感知（Perception）** | 规划中 | 传感器与场景理解 |
| **Agent** | 规划中 | 任务编排与决策 |
| **仿真** | 脚手架 | [`crates/simulation`](crates/simulation/) |

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
- 仿真：Python + MuJoCo，见 [`crates/simulation/README.md`](crates/simulation/README.md)

## 构建

在仓库根目录：

```bash
cargo build
```

## CI

Push 与 Pull Request 会运行 [GitHub Actions](.github/workflows/ci.yml)：`cargo test --workspace`、编译全部 runtime 示例，并 smoke 运行 `client` 示例。

**PR 合并前须 CI 通过**：在 GitHub 仓库 **Settings → Branches → Branch protection rules** 中为目标分支（如 `main`）启用 **Require status checks to pass**，并勾选 **`test and examples`**（或 `CI / test and examples`）。

## 依赖（应用）

```toml
zos = { path = "crates/zos" }
```

## Git hooks（可选）

去掉 Cursor 自动写入的 `Co-authored-by` 行：

```bash
git config core.hooksPath .githooks
```

## 许可证

Apache-2.0；源码文件含 SPDX 头。仿真资源许可见 [`crates/simulation/README.md`](crates/simulation/README.md)。
