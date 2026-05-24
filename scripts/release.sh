#!/usr/bin/env bash
#
# ClawHeart Desktop · 一键发版脚本
#
# 调度：本地 mac × 2 平台 build  +  GitHub Actions Windows CI
# 自动：bump 版本 → commit → push → 触发 win CI → 本地 mac build (并行)
#       → 等 CI → 下载 win exe → 上传所有 dmg/exe + 3 个稳定别名 → 可选 publish
#
# 用法:
#   scripts/release.sh <version>            # 创建 draft Release（默认）
#   scripts/release.sh <version> --publish  # 一键 publish 为 latest
#
# 例:
#   scripts/release.sh 2.0.0-alpha.2
#   scripts/release.sh 2.0.0 --publish

set -euo pipefail

# ──────────────────────────────────────────────────────────────────
# 解析参数
# ──────────────────────────────────────────────────────────────────
VERSION="${1:-}"
DO_PUBLISH=false
if [ "${2:-}" = "--publish" ]; then
  DO_PUBLISH=true
fi

if [ -z "$VERSION" ]; then
  echo "用法: $0 <version> [--publish]"
  echo "例:   $0 2.0.0-alpha.2"
  exit 1
fi

# 标准化版本号（去掉 v 前缀）
VERSION="${VERSION#v}"
TAG="v$VERSION"
REPO="tjsdyy/clawheartv2"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# ANSI colors
G="\033[32m"; B="\033[34m"; Y="\033[33m"; R="\033[31m"; C="\033[36m"; X="\033[0m"

log()  { printf "${B}▶${X} %s\n" "$*"; }
ok()   { printf "${G}✓${X} %s\n" "$*"; }
warn() { printf "${Y}⚠${X} %s\n" "$*"; }
err()  { printf "${R}✗${X} %s\n" "$*"; exit 1; }

# ──────────────────────────────────────────────────────────────────
# Precheck
# ──────────────────────────────────────────────────────────────────
log "Precheck"
cd "$ROOT"

[ "$(uname -s)" = "Darwin" ] || err "本脚本仅在 macOS 运行（mac 双架构本地 build）"

command -v gh    >/dev/null || err "缺少 gh CLI（brew install gh）"
command -v pnpm  >/dev/null || err "缺少 pnpm（npm i -g pnpm）"
command -v cargo >/dev/null || err "缺少 cargo（rustup 安装）"
command -v jq    >/dev/null || warn "建议安装 jq（brew install jq）"

# 工作目录干净
if [ -n "$(git status --porcelain)" ]; then
  err "工作树有未提交改动，请先 commit/stash"
fi

# rustup targets
for t in aarch64-apple-darwin x86_64-apple-darwin; do
  if ! rustup target list --installed | grep -q "$t"; then
    log "安装 rustup target $t"
    rustup target add "$t"
  fi
done

# 当前分支
BRANCH=$(git rev-parse --abbrev-ref HEAD)
[ "$BRANCH" = "main" ] || warn "当前分支不是 main（是 $BRANCH），继续？(Enter 继续 / Ctrl-C 取消)" && read -r

ok "环境就绪 · tag=$TAG · publish=$DO_PUBLISH"

# ──────────────────────────────────────────────────────────────────
# 1. Bump 三处版本号
# ──────────────────────────────────────────────────────────────────
log "Bump 版本号到 $VERSION"

sed -i.bak "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" \
  package.json src-tauri/tauri.conf.json
sed -i.bak "s/^version = \"[^\"]*\"$/version = \"$VERSION\"/" \
  src-tauri/Cargo.toml
rm -f package.json.bak src-tauri/tauri.conf.json.bak src-tauri/Cargo.toml.bak

# 校验
for f in package.json src-tauri/tauri.conf.json; do
  grep -q "\"version\": \"$VERSION\"" "$f" || err "$f 版本号没改对"
done
grep -q "^version = \"$VERSION\"$" src-tauri/Cargo.toml || err "Cargo.toml 版本号没改对"

# Cargo.lock 跟随更新
cargo update -p clawheart --offline 2>/dev/null || true

git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "release: $TAG"
git push origin "$BRANCH"

ok "版本号已 bump + push（commit $(git rev-parse --short HEAD)）"

# ──────────────────────────────────────────────────────────────────
# 2. 触发 Windows CI（异步）
# ──────────────────────────────────────────────────────────────────
log "触发 GitHub Actions Windows build"

gh workflow run release-windows-only.yml -R "$REPO" -f "release_tag=$TAG"

# 等 GitHub 入队
sleep 5
WIN_RUN_ID=$(gh run list -R "$REPO" \
  --workflow release-windows-only.yml --limit 1 \
  --json databaseId -q '.[0].databaseId')

ok "Windows CI 已启动: https://github.com/$REPO/actions/runs/$WIN_RUN_ID"

# ──────────────────────────────────────────────────────────────────
# 3. 本地并行 build：mac arm64 + mac x64
# ──────────────────────────────────────────────────────────────────
log "本地双架构 build（mac arm64 + mac x64）— 串行跑避免 cargo lock 冲突"

# 两个 target 共用 src-tauri/target 父目录但各自子目录，并行会引发
# pnpm 前端 build 重叠 / cargo metadata 锁竞争。串行更稳。

LOG_ARM="/tmp/clawheart-release-arm64.log"
LOG_X64="/tmp/clawheart-release-x64.log"

log "  build aarch64-apple-darwin → 日志 $LOG_ARM"
if ! pnpm tauri build --target aarch64-apple-darwin > "$LOG_ARM" 2>&1; then
  err "mac arm64 build 失败，最后 30 行日志:
$(tail -30 "$LOG_ARM")"
fi
ok "  mac arm64 完成"

log "  build x86_64-apple-darwin → 日志 $LOG_X64"
if ! pnpm tauri build --target x86_64-apple-darwin > "$LOG_X64" 2>&1; then
  err "mac x64 build 失败，最后 30 行日志:
$(tail -30 "$LOG_X64")"
fi
ok "  mac x64 完成"

# ── 本地构建 clawheart-cli 双架构（Phase 7）────────────────────────
log "本地构建 clawheart-cli（mac arm64 + x64）"

LOG_CLI_ARM="/tmp/clawheart-release-cli-arm64.log"
LOG_CLI_X64="/tmp/clawheart-release-cli-x64.log"

if ! cargo build --manifest-path src-tauri/Cargo.toml \
     --release --no-default-features --features cli \
     --bin clawheart-cli --target aarch64-apple-darwin \
     > "$LOG_CLI_ARM" 2>&1; then
  err "CLI arm64 build 失败：$(tail -20 "$LOG_CLI_ARM")"
fi
if ! cargo build --manifest-path src-tauri/Cargo.toml \
     --release --no-default-features --features cli \
     --bin clawheart-cli --target x86_64-apple-darwin \
     > "$LOG_CLI_X64" 2>&1; then
  err "CLI x64 build 失败：$(tail -20 "$LOG_CLI_X64")"
fi
ok "  clawheart-cli 双架构完成"

ARM_DMG="$ROOT/src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/ClawHeart_${VERSION}_aarch64.dmg"
X64_DMG="$ROOT/src-tauri/target/x86_64-apple-darwin/release/bundle/dmg/ClawHeart_${VERSION}_x64.dmg"

[ -f "$ARM_DMG" ] || err "找不到 $ARM_DMG"
[ -f "$X64_DMG" ] || err "找不到 $X64_DMG"

ARM_SIZE=$(du -h "$ARM_DMG" | cut -f1)
X64_SIZE=$(du -h "$X64_DMG" | cut -f1)
ok "本地 dmg 就绪: arm64=$ARM_SIZE  x64=$X64_SIZE"

# ──────────────────────────────────────────────────────────────────
# 4. 等待 Windows CI
# ──────────────────────────────────────────────────────────────────
log "等待 Windows CI 完成…"

if ! gh run watch "$WIN_RUN_ID" -R "$REPO" --exit-status; then
  err "Windows CI 失败，查看 https://github.com/$REPO/actions/runs/$WIN_RUN_ID"
fi

ok "Windows CI 完成"

# ──────────────────────────────────────────────────────────────────
# 5. 上传 mac dmg + 3 个稳定别名
# ──────────────────────────────────────────────────────────────────
log "上传 mac dmg 到 Release"

gh release upload "$TAG" -R "$REPO" --clobber "$ARM_DMG" "$X64_DMG"
ok "mac dmg 已上传"

log "生成 3 个稳定别名"

STAGE="$(mktemp -d -t clawheart-aliases)"
cp "$ARM_DMG" "$STAGE/ClawHeart_aarch64.dmg"
cp "$X64_DMG" "$STAGE/ClawHeart_x64.dmg"

# win exe 在 release 上，下载一份重命名再上传
WIN_EXE_VERSIONED="ClawHeart_${VERSION}_x64-setup.exe"
gh release download "$TAG" -R "$REPO" -p "$WIN_EXE_VERSIONED" --output "$STAGE/$WIN_EXE_VERSIONED"
cp "$STAGE/$WIN_EXE_VERSIONED" "$STAGE/ClawHeart_x64-setup.exe"

gh release upload "$TAG" -R "$REPO" --clobber \
  "$STAGE/ClawHeart_aarch64.dmg" \
  "$STAGE/ClawHeart_x64.dmg" \
  "$STAGE/ClawHeart_x64-setup.exe"

# ── CLI binary tarball（mac arm64 + x64）+ 稳定别名 ──
log "打包 + 上传 clawheart-cli tarball"
ARM_CLI_BIN="$ROOT/src-tauri/target/aarch64-apple-darwin/release/clawheart-cli"
X64_CLI_BIN="$ROOT/src-tauri/target/x86_64-apple-darwin/release/clawheart-cli"

tar -czf "$STAGE/clawheart-cli-${VERSION}-aarch64-apple-darwin.tar.gz" \
  -C "$(dirname "$ARM_CLI_BIN")" clawheart-cli
tar -czf "$STAGE/clawheart-cli-${VERSION}-x86_64-apple-darwin.tar.gz" \
  -C "$(dirname "$X64_CLI_BIN")" clawheart-cli

# 稳定别名（去版本号）
cp "$STAGE/clawheart-cli-${VERSION}-aarch64-apple-darwin.tar.gz" \
   "$STAGE/clawheart-cli-aarch64-apple-darwin.tar.gz"
cp "$STAGE/clawheart-cli-${VERSION}-x86_64-apple-darwin.tar.gz" \
   "$STAGE/clawheart-cli-x86_64-apple-darwin.tar.gz"

gh release upload "$TAG" -R "$REPO" --clobber \
  "$STAGE/clawheart-cli-${VERSION}-aarch64-apple-darwin.tar.gz" \
  "$STAGE/clawheart-cli-${VERSION}-x86_64-apple-darwin.tar.gz" \
  "$STAGE/clawheart-cli-aarch64-apple-darwin.tar.gz" \
  "$STAGE/clawheart-cli-x86_64-apple-darwin.tar.gz"

# Windows CLI zip 已由 release-windows-only.yml 上传，这里只补稳定别名
WIN_CLI_ZIP_VERSIONED="clawheart-cli-${VERSION}-x86_64-pc-windows-msvc.zip"
if gh release download "$TAG" -R "$REPO" -p "$WIN_CLI_ZIP_VERSIONED" --output "$STAGE/$WIN_CLI_ZIP_VERSIONED" 2>/dev/null; then
  cp "$STAGE/$WIN_CLI_ZIP_VERSIONED" "$STAGE/clawheart-cli-x86_64-pc-windows-msvc.zip"
  gh release upload "$TAG" -R "$REPO" --clobber \
    "$STAGE/clawheart-cli-x86_64-pc-windows-msvc.zip"
fi

rm -rf "$STAGE"
ok "稳定别名 + CLI tarball 已上传"

# ──────────────────────────────────────────────────────────────────
# 6. （可选）一键 publish
# ──────────────────────────────────────────────────────────────────
if $DO_PUBLISH; then
  log "publish Release 为 latest"
  gh release edit "$TAG" -R "$REPO" --draft=false --prerelease=false
  ok "已 publish: https://github.com/$REPO/releases/tag/$TAG"
else
  ok "Release 仍为 draft，可去 https://github.com/$REPO/releases 检查后 publish"
  echo ""
  echo "   下次想直接 publish 加 --publish 标志："
  echo "   $0 $VERSION --publish"
fi

# ──────────────────────────────────────────────────────────────────
# 7. Summary
# ──────────────────────────────────────────────────────────────────
echo ""
printf "${C}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${X}\n"
printf "${G}✓${X} ClawHeart Desktop $TAG 发布完成\n"
echo ""
echo "  📦 Release:  https://github.com/$REPO/releases/tag/$TAG"
echo "  🌐 官网:     https://clawheart.live （≤10 分钟 ISR 自动同步）"
echo ""
echo "  稳定下载链接（永久）："
echo "    https://github.com/$REPO/releases/latest/download/ClawHeart_aarch64.dmg"
echo "    https://github.com/$REPO/releases/latest/download/ClawHeart_x64.dmg"
echo "    https://github.com/$REPO/releases/latest/download/ClawHeart_x64-setup.exe"
printf "${C}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${X}\n"
