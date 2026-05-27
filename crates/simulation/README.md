# Simulation (MuJoCo)

Unitree **Go2** 四足模型来自 [mujoco_menagerie](https://github.com/google-deepmind/mujoco_menagerie)（BSD-3-Clause）。

## 准备

```bash
pip install mujoco
bash fetch_go2.sh   # 下载 unitree_go2/（约 29MB，已 gitignore）
```

## 运行

```bash
python main.py
```

加载 `unitree_go2/scene.xml`（含地面与光照）。
