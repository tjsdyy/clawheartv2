#!/usr/bin/env bash
# 从 public/clawheart.svg 生成 tauri 所需的 icon 集

set -euo pipefail
cd "$(dirname "$0")/.."

if ! command -v rsvg-convert >/dev/null 2>&1 && ! command -v inkscape >/dev/null 2>&1; then
  echo "需要 rsvg-convert 或 inkscape 来把 SVG 转 PNG"
  case "$(uname -s)" in
    Darwin*) echo "  macOS: brew install librsvg" ;;
    Linux*)  echo "  Linux: apt-get install librsvg2-bin" ;;
  esac
  exit 1
fi

# 生成 1024x1024 PNG 作为 tauri icon 生成的输入
tmp_png=$(mktemp -t clawheart-icon-XXXXXX.png)
if command -v rsvg-convert >/dev/null 2>&1; then
  rsvg-convert -w 1024 -h 1024 public/clawheart.svg -o "$tmp_png"
else
  inkscape public/clawheart.svg -w 1024 -h 1024 -o "$tmp_png"
fi

pnpm tauri icon "$tmp_png"
rm -f "$tmp_png"

echo "✓ Icons generated in src-tauri/icons/"
