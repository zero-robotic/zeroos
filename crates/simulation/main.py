from pathlib import Path

import mujoco
import mujoco.viewer

SIM_DIR = Path(__file__).resolve().parent
# Unitree Go2 from MuJoCo Menagerie (ground plane + lighting + robot)
SCENE_PATH = SIM_DIR / "unitree_go2" / "scene.xml"


def main() -> None:
    if not SCENE_PATH.is_file():
        raise SystemExit(
            f"未找到 Go2 模型: {SCENE_PATH}\n"
            "请先运行: bash fetch_go2.sh"
        )

    print("正在加载 Unitree Go2 四足机器人模型 (MuJoCo Menagerie)...")
    model = mujoco.MjModel.from_xml_path(str(SCENE_PATH))
    data = mujoco.MjData(model)

    print("成功！正在拉起原生 3D 交互界面...")
    print("提示：左键拖拽机器人，右键旋转视角，滚轮缩放。")

    mujoco.viewer.launch(model, data)

    print("窗口已关闭，仿真结束。")


if __name__ == "__main__":
    main()
