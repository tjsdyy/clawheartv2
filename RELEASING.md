# 发版流程

三条路径，按方便度从高到低：

| 路径 | 适合场景 | 速度 | 备注 |
|---|---|---|---|
| **A. 一键脚本** | 日常发版 | ~10 分钟 | mac 本地 + win CI，自动 publish |
| **B. push tag** | 离线时 / 不想本地等 | ~20 分钟 | 全平台走 CI，macos-13 慢 |
| **C. 网页手动 dispatch** | 临时实验 / 跳版本号 | ~20 分钟 | 不需要本地环境 |

## A. 一键脚本（推荐）

```bash
# 1. draft 模式 — 创建 draft Release，最后手动 publish
scripts/release.sh 2.0.0-alpha.2

# 2. 直接 publish 模式 — 跑完自动 publish 为 latest
scripts/release.sh 2.0.0-alpha.2 --publish
```

脚本会自动：

1. precheck（工作树干净、gh / pnpm / cargo 都装好）
2. bump 三处版本号（`package.json` / `tauri.conf.json` / `Cargo.toml`）+ commit + push
3. 触发 GitHub Actions `release-windows-only.yml`（Windows 在 CI 上跑）
4. **同时**在本地跑 `mac aarch64 + mac x86_64` 双架构 build
5. 等 Windows CI 完成 → 下载 exe
6. 上传所有产物 + 生成 3 个稳定别名
7. （可选）publish 为 latest

预计耗时 ~10 分钟：本地 mac arm64 build ~3min · mac x64 ~5min · Windows CI ~7min（并行）。

### 前置要求

- macOS（本脚本只在 mac 跑）
- `gh` CLI 已登录 GitHub（`gh auth login`）
- `pnpm` + `cargo` 已装
- rustup target `aarch64-apple-darwin` + `x86_64-apple-darwin`（脚本会自动 add）
- 工作目录干净（无 uncommitted）

## B. push tag（全 CI）

```bash
# 1. bump 三处版本号
vim src-tauri/tauri.conf.json  # 改 "version"
vim package.json               # 改 "version"
vim src-tauri/Cargo.toml       # 改 version = "..."

# 2. commit + tag + push
git add -A
git commit -m "release: v2.0.0-alpha.2"
git tag v2.0.0-alpha.2
git push origin main --tags
```

push tag 后 `.github/workflows/release.yml` 自动跑：
- macos-arm64（macos-14 runner）
- macos-x64（macos-13 runner，**慢**）
- windows-x64（windows-latest）
- `finalize-aliases` — 加去版本号的稳定别名

构建产物会出现在 GitHub Release **draft** 状态。然后手动 Publish。

## C. 网页手动 dispatch

去 https://github.com/tjsdyy/clawheartv2/actions/workflows/release.yml/dispatches

- `mode = release`
- `release_tag = v2.0.0-alpha.2`

效果同 B，跑完后手动 Publish。

适合：测试 CI 流程；或者想发版但不想 push tag。

---

## 发布到正式

draft Release 需要人工 publish：

1. 打开 https://github.com/tjsdyy/clawheartv2/releases
2. 检查 draft 中的 6 个产物：
   - `ClawHeart_aarch64.dmg` + `ClawHeart_<ver>_aarch64.dmg`
   - `ClawHeart_x64.dmg` + `ClawHeart_<ver>_x64.dmg`
   - `ClawHeart_x64-setup.exe` + `ClawHeart_<ver>_x64-setup.exe`
3. 编辑 Release notes（自动模板已填充 commit，补 changelog）
4. 点击 **Publish release**

或者用 gh CLI：

```bash
gh release edit v2.0.0-alpha.2 --draft=false --prerelease=false
```

如用路径 A 的 `--publish` flag，这步已自动完成。

## 官网的稳定 URL

`clawheart.live` 用以下三个永久链接：

| 平台 | URL |
|---|---|
| macOS Apple Silicon | `https://github.com/tjsdyy/clawheartv2/releases/latest/download/ClawHeart_aarch64.dmg` |
| macOS Intel | `…/releases/latest/download/ClawHeart_x64.dmg` |
| Windows x64 | `…/releases/latest/download/ClawHeart_x64-setup.exe` |

`releases/latest/download/X` 是 GitHub 提供的稳定别名，永远 302 跳到当前 latest Release 的 asset，**官网零改动**。

Vercel ISR 缓存 10 分钟，publish 后 10 分钟内官网自动同步。

## 国内 CDN 同步（可选）

GitHub Release 在国内访问慢。如需镜像到腾讯云 COS / 自有服务器：

**手动 mirror**：

```bash
for f in ClawHeart_aarch64.dmg ClawHeart_x64.dmg ClawHeart_x64-setup.exe; do
  curl -L -o "/tmp/$f" \
    "https://github.com/tjsdyy/clawheartv2/releases/latest/download/$f"
  coscli cp "/tmp/$f" "cos://<bucket>/clawheart/$f"
done
```

**CI 自动 mirror**：参考 [`scripts/release.sh`](scripts/release.sh) 模式，加一个 mirror-to-cos step。需要在 GitHub repo Settings → Secrets 配置：
- `TENCENT_SECRET_ID`
- `TENCENT_SECRET_KEY`
- `COS_BUCKET_NAME`
- `COS_REGION`

镜像就绪后，在 Netlify / Vercel 项目设置中加 env 覆盖默认 URL：

```
VITE_DESKTOP_DOWNLOAD_MAC_ARM64_URL=https://<bucket>.cos.<region>.myqcloud.com/clawheart/ClawHeart_aarch64.dmg
VITE_DESKTOP_DOWNLOAD_MAC_INTEL_URL=…
VITE_DESKTOP_DOWNLOAD_WIN_URL=…
```

## 故障排查

| 现象 | 原因 / 解决 |
|---|---|
| 脚本报「工作树有未提交改动」 | `git stash` 或先 commit |
| 脚本报「gh CLI 缺 workflow scope」 | `gh auth refresh -h github.com -s workflow` |
| Windows MSI bundle 失败"pre-release must be numeric-only" | 已修复 — `tauri.conf.json` `bundle.targets` 排除 `msi`，只用 nsis |
| draft Release 没创建 | 检查 push 的 tag 是否匹配 `v*` 模式（必须以 `v` 开头） |
| dmg 在 macOS 提示「来自身份不明的开发者」 | 当前未启用 codesign + notarize；正式发版需配置 Apple Developer 证书 + `APPLE_*` secrets |
| Windows exe SmartScreen 警告 | 未签名 EV cert；当前 alpha 阶段已知问题 |

## 版本号约定

- alpha 阶段：`v2.0.0-alpha.N`（N=0,1,2...）
- 正式 minor：`v2.1.0`
- 补丁：`v2.0.1`

务必保证 `src-tauri/tauri.conf.json::version` 与 tag 名（去掉 `v`）一致。脚本会自动保证三处同步，手动发版时需自己注意。
