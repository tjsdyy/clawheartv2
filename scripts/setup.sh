#!/usr/bin/env bash
# 一键开发环境检查 + 设置

set -euo pipefail

cd "$(dirname "$0")/.."

red()   { printf "\033[31m%s\033[0m\n" "$*"; }
green() { printf "\033[32m%s\033[0m\n" "$*"; }
yellow(){ printf "\033[33m%s\033[0m\n" "$*"; }

green "▶ ClawHeart v2 setup check"

# Rust
if ! command -v rustc >/dev/null 2>&1; then
  red "✗ Rust 未安装。运行：curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  exit 1
fi
green "✓ Rust $(rustc --version)"

# Node
if ! command -v node >/dev/null 2>&1; then
  red "✗ Node 未安装。建议 nvm + node 20.x"
  exit 1
fi
node_major=$(node -v | sed -E 's/v([0-9]+)\..*/\1/')
if [ "$node_major" -lt 20 ]; then
  yellow "⚠ Node 版本 $(node -v) < 20，建议升级"
fi
green "✓ Node $(node -v)"

# pnpm
if ! command -v pnpm >/dev/null 2>&1; then
  yellow "⚠ pnpm 未安装，正在用 corepack 启用…"
  corepack enable
fi
green "✓ pnpm $(pnpm -v)"

# Tauri 系统依赖
case "$(uname -s)" in
  Linux*)
    yellow "ℹ Linux 平台需要：libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libssl-dev"
    ;;
  Darwin*)
    if ! xcode-select -p >/dev/null 2>&1; then
      red "✗ macOS 需要 Xcode CLT，运行：xcode-select --install"
      exit 1
    fi
    green "✓ Xcode CLT $(xcode-select -p)"
    ;;
esac

# 安装前端依赖
yellow "ℹ 安装前端依赖（pnpm install）…"
pnpm install

# 拉取 Rust 依赖
yellow "ℹ 拉取 Rust 依赖（cargo fetch）…"
(cd src-tauri && cargo fetch)

green "✓ 环境就绪。运行 \`pnpm tauri:dev\` 启动应用。"
