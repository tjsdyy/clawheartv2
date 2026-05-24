#!/usr/bin/env bash
#
# ClawHeart Skill 跨 Agent 安装器
# 把 SKILL.md 拷贝到本机所有发现的 AI Agent skills 目录
#
# 用法:
#   ./install.sh                  # 自动检测 + dry-run 列表
#   ./install.sh --confirm        # 真实拷贝
#   ./install.sh --confirm --to=claude,codex   # 仅安装到指定 agent

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SKILL_SRC="$SCRIPT_DIR/clawheart-security"

if [ ! -f "$SKILL_SRC/SKILL.md" ]; then
  echo "✗ 找不到 SKILL.md: $SKILL_SRC/SKILL.md" >&2
  exit 1
fi

CONFIRM=false
ONLY_LIST=""
for arg in "$@"; do
  case "$arg" in
    --confirm) CONFIRM=true ;;
    --to=*)    ONLY_LIST="${arg#--to=}" ;;
    -h|--help)
      echo "用法: $0 [--confirm] [--to=claude,codex,cursor,openeva,openclaw,clawcode]"
      exit 0 ;;
    *)
      echo "未知参数 $arg" >&2
      exit 1 ;;
  esac
done

# 列出 ~/.<agent>/ 中 skill 安装目标候选
detect_targets() {
  for d in "$HOME"/.*; do
    [ -d "$d" ] || continue
    name="$(basename "$d")"
    name="${name#.}"
    # 跳过系统 dotfile + agents（SSOT 集中库，不是 Agent 本身）
    case "$name" in
      git|github|gitignore|vscode|idea|cache|cargo|rustup|npm|nvm|\
        bashrc|zshrc|bash_history|zsh_history|profile|ssh|gnupg|\
        local|config|docker|kube|DS_Store|Trash|CFUserTextEncoding|\
        agents|clawheart-v2) continue ;;
    esac
    [ -n "$name" ] || continue
    skills_dir="$d/skills"
    # 只在 ~/.<agent>/skills/ 已存在时安装（说明该 agent 真的支持 skills）
    [ -d "$skills_dir" ] || continue
    echo "$name|$skills_dir"
  done
}

matches_filter() {
  local agent="$1"
  [ -z "$ONLY_LIST" ] && return 0
  IFS=',' read -ra arr <<< "$ONLY_LIST"
  for a in "${arr[@]}"; do
    [ "$a" = "$agent" ] && return 0
  done
  return 1
}

targets=()
while IFS= read -r line; do
  targets+=("$line")
done < <(detect_targets)

if [ ${#targets[@]} -eq 0 ]; then
  echo "未检测到任何带 skills/ 目录的 Agent"
  echo "若你的 Agent 用其他路径，手动拷贝 $SKILL_SRC 到对应目录即可"
  exit 0
fi

echo "✓ 检测到 ${#targets[@]} 个 Agent 候选："
echo ""
printf "%-15s  %s\n" "AGENT" "TARGET"
printf "%-15s  %s\n" "-----" "------"
selected=()
for line in "${targets[@]}"; do
  agent="${line%%|*}"
  skills_dir="${line#*|}"
  if matches_filter "$agent"; then
    printf "%-15s  %s/clawheart-security/\n" "$agent" "$skills_dir"
    selected+=("$line")
  else
    printf "%-15s  %s (跳过 · 未在 --to 列表)\n" "$agent" "$skills_dir"
  fi
done
echo ""

if ! $CONFIRM; then
  echo "ℹ DRY RUN —— 加 --confirm 真实安装。"
  exit 0
fi

# 真实拷贝
for line in "${selected[@]}"; do
  agent="${line%%|*}"
  skills_dir="${line#*|}"
  dest="$skills_dir/clawheart-security"
  mkdir -p "$dest"
  cp -R "$SKILL_SRC"/* "$dest/"
  echo "✓ 已安装到 $dest"
done

echo ""
echo "✓ 完成。在对应 Agent 中可用以下示例触发："
echo "  - 帮我扫一下 AI 安全风险"
echo "  - 我装了哪些 Agent"
echo "  - 看看 ClawHeart 整机状态"
