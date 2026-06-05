#!/usr/bin/env bash
# Download Unitree Go2 MJCF from google-deepmind/mujoco_menagerie (BSD-3-Clause).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

if [[ -f unitree_go2/scene.xml ]]; then
  echo "unitree_go2 已存在，跳过下载。"
  exit 0
fi

ARCHIVE="${ROOT}/.menagerie.tar.gz"
echo "正在下载 mujoco_menagerie (unitree_go2)..."
curl -fsSL "https://github.com/google-deepmind/mujoco_menagerie/archive/refs/heads/main.tar.gz" -o "$ARCHIVE"
tar xzf "$ARCHIVE" "mujoco_menagerie-main/unitree_go2"
mv "mujoco_menagerie-main/unitree_go2" .
rm -rf "mujoco_menagerie-main" "$ARCHIVE"
echo "完成: ${ROOT}/unitree_go2"
