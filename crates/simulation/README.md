# Simulation (MuJoCo)

平面**差分轮**小车仿真，用于后续定位 / 导航联调（`cmd_vel`、里程计、激光等）。

机器人在 [`robots/`](robots/) 目录下，默认加载差分轮 [`robots/diff_drive/scene.xml`](robots/diff_drive/scene.xml)，无需额外下载。

## 准备

### MuJoCo 库

`zos-simulation` 通过 [mujoco-rs](https://github.com/davidhozic/mujoco-rs) 链接 MuJoCo **3.3.7**（与 `kcking/tap/mujoco@3.7` 一致）。

**Linux / Windows**（自动下载 MuJoCo）：

```bash
export MUJOCO_DOWNLOAD_DIR="$HOME/libraries"   # 会在此目录下创建 mujoco-x.y.z/
cargo run -p zos-simulation --features auto-mujoco
```

**macOS**（推荐 Homebrew，pkg-config 自动发现）：

```bash
brew install kcking/tap/mujoco@3.7
export PKG_CONFIG_PATH="$(brew --prefix mujoco@3.7)/lib/pkgconfig"
cargo run -p zos-simulation
```

`PKG_CONFIG_PATH` 需在**编译时**设置（供 `mujoco-rs` 发现 MuJoCo；`build.rs` 同时把 rpath 写入可执行文件）。若报 `Library not loaded: @rpath/mujoco.framework`，执行 `cargo clean -p zos-simulation` 后重新 `cargo run`。

macOS 上 `OpenGL ARB_framebuffer_object required` 的根因是 **glutin** 在 CGL 上强制 OpenGL profile（[glutin#1740](https://github.com/rust-windowing/glutin/pull/1740)，尚未进 crates.io）。根 `Cargo.toml` 的 `[patch.crates-io]` 临时指向含修复的 git 版 glutin。官方合并发版后可删该 patch。若仍报错：

```bash
cargo clean -p mujoco-rs -p zos-simulation
export PKG_CONFIG_PATH="$(brew --prefix mujoco@3.7)/lib/pkgconfig"
cargo run -p zos-simulation
```

也可从 [MuJoCo 发布页](https://github.com/google-deepmind/mujoco/releases) 手动安装并设置 `MUJOCO_DYNAMIC_LINK_DIR`，参见 [mujoco-rs 安装说明](https://mujoco-rs.readthedocs.io/en/v2.3.x/installation.html)。

## 运行

```bash
cargo run -p zos-simulation
```

加载 `robots/diff_drive/scene.xml`（场景 + `include` 的 [`robot.xml`](robots/diff_drive/robot.xml)）。

**键盘遥控**：按住 `W`/`S` 前进后退，按住 `A`/`D` 原地转向，松开即停。窗口内「遥控」面板显示当前 `cmd_vel`。

## 模型结构

```
robots/diff_drive/
├── robot.xml                        # 差分底盘、传感器位、执行器
├── scene.xml                        # 办公场景 + include robot / office_floor
└── environments/
    └── office_floor.xml             # 开放式办公区（≈20 m × 16 m）
```

默认场景为**现代开放式办公区**（吊顶、窗景、玻璃隔断、复合工位家具），包含：

- 室内照明：仅顶部点光源（无可视灯盘/吊顶几何体）
- 开放式平面布局（无单独中央走廊）
- 南北采光：窗框 + 半透明玻璃幕墙
- 功能分区：会议室（实木会议桌、座椅、白板）、经理室、开放工位、L 形前台、沙发区、仓储货架、复印机
- 工位细节：桌板 + 桌腿 + 显示器 + 办公椅；矮墙 + 上部玻璃隔断

适合后续激光建图 / 导航联调。机器人初始位置在原点附近，南侧留有门洞通行。

坐标系约定（对齐 ROS）：**+X 车头前进**，+Y 左侧，+Z 向上。

| 参数 / 坐标系 | 值 |
|------|-----|
| 轮半径 | 0.05 m |
| 轮距 | 0.32 m |
| `base_link` | 底盘根坐标系 |
| `imu_link` | IMU site（`imu_accel` / `imu_gyro`） |
| `lidar_link` | 2D 激光安装位（后续 `mj_ray` 扫描） |
| `camera_link` | 前置相机（`front_camera`） |
| 执行器 | 底盘平面 `velocity`（`drive_x` / `drive_y` / `drive_yaw`） |

外观：橙色前保险杠 + 绿色前灯 / 灰色后保险杠 + 红色后灯，便于区分车头车尾。

与 [`src/lib.rs`](src/lib.rs) 中 `WHEEL_RADIUS` / `WHEEL_BASE` 及 `read_odom()` / `read_imu()` 保持一致。遥控默认 `cmd_vel`：线速度 **0.8 m/s**、角速度 **2.0 rad/s**（可在 `SPEED_LINEAR` / `SPEED_ANGULAR` 调整，执行器上限 2 / 3）。

## 机器人目录

| 目录 | 说明 |
|------|------|
| [`robots/diff_drive/`](robots/diff_drive/) | 差分轮小车（当前默认） |
| [`robots/humanoid/`](robots/humanoid/) | 人形模型（备用，未接入 `main`） |

## CI / 无头编译

无显示环境仅做库编译与运动学测试（不构建 viewer 二进制）：

```bash
cargo test -p zos-simulation --no-default-features
```

## 后续扩展

- 2D 激光：以 `lidar_link` 为原点，`mj_ray` 批量扫描
- 相机：`front_camera` + offscreen `Renderer`
- 与 `zos-runtime`：Zenoh 订阅 `cmd_vel`、发布 `read_odom()` / `read_imu()` / `scan`
