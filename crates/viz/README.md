# zos-viz

基于 [`zos-runtime`](../runtime/) 的桌面消息可视化工具：通过 Zenoh 订阅话题，用 egui 展示实时数据。本机或局域网内另一台机器运行均可（共享 Zenoh 网络）。

## 传输

| 数据 | 传输 |
|------|------|
| `cmd_vel`、`odom`、`scan` 等话题 | **zos-runtime / Zenoh** |
| 相机（后续） | Runtime 压缩帧（桌面）；浏览器场景再考虑 WebRTC |

## 运行

先启动发布者（示例）：

```bash
cargo run -p zos-runtime --example pub_sub
```

另开终端启动可视化：

```bash
cargo run -p zos-viz
```

自定义 Zenoh 配置（跨主机）时，在发布者与 `zos-viz` 两侧使用相同的 `zenoh.json5`，并在代码中改为 `init_from_file`（后续可加 CLI）。

## 当前订阅

| 话题 | 类型 | 状态 |
|------|------|------|
| `cmd_vel` | `zos.geometry.Twist` | 已实现 |
| `scan` | `LaserScan`（待定义） | 占位 |
| `camera` | `CompressedImage`（待定义） | 占位 |

## CI / 无头

```bash
cargo test -p zos-viz --no-default-features
cargo build -p zos-viz --no-default-features
```

带 UI 的完整构建：`cargo build -p zos-viz`（需图形环境）。
