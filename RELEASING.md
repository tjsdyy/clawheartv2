# 发版流程

ClawHeart Desktop v2 的自动化发版基于 GitHub Actions
[`.github/workflows/release.yml`](.github/workflows/release.yml)，三平台并行构建并上传到 GitHub Release。

## 一次完整发版

```bash
# 1. 在 src-tauri/tauri.conf.json 中 bump 版本号
#    "version": "2.0.0-alpha.0" → "2.0.0-alpha.1"
#    （同时建议同步 src-tauri/Cargo.toml 与 package.json 的版本，便于追溯）

# 2. 提交并打 tag
git add -A
git commit -m "release: v2.0.0-alpha.1"
git tag v2.0.0-alpha.1
git push origin main --tags
```

push tag 后自动触发：
- `macos-arm64` (macos-14 runner)
- `macos-x64`   (macos-13 runner)
- `windows-x64` (windows-latest runner)
- `finalize-aliases` — 给产物加去版本号的稳定别名

构建产物会出现在 GitHub Release **draft** 状态。

## 发布到正式

1. 打开 https://github.com/tjsdyy/clawheartv2/releases
2. 检查 draft 中的 4 个产物：
   - `ClawHeart_aarch64.dmg` + `ClawHeart_<ver>_aarch64.dmg`
   - `ClawHeart_x64.dmg` + `ClawHeart_<ver>_x64.dmg`
   - `ClawHeart_x64-setup.exe` + `ClawHeart_<ver>_x64-setup.exe`
3. 编辑 Release notes（自动模板已填充 commit 信息，可补 changelog）
4. 点击 **Publish release**

发布完成后，下载页 URL 自动指向新版本（官网零改动）。

## 官网的稳定 URL

`opencarapace-server/frontend/src/pages/DownloadPage.tsx` 默认指向：

```
https://github.com/tjsdyy/clawheartv2/releases/latest/download/<filename>
```

| 平台 | 稳定文件名 |
|------|-----------|
| macOS Apple Silicon | `ClawHeart_aarch64.dmg` |
| macOS Intel | `ClawHeart_x64.dmg` |
| Windows x64 | `ClawHeart_x64-setup.exe` |

`releases/latest/download/<x>` 是 GitHub 提供的稳定别名，永远 302 跳到当前 latest Release 的 asset，无需手动更新前端。

## 手动验证（不发版）

在 Actions 页面手动触发 workflow，勾 `dry_run = true`：
- 跑完整构建链
- **不**创建 Release
- 产物作为 Actions artifact 留存 14 天，可下载验证

适合用于：CI 配置调试、依赖升级前的回归。

## 国内 CDN 同步（可选）

GitHub Release 在国内访问慢。如需把产物镜像到腾讯云 COS / anxin.anakkix.cn，可在 publish 后用以下方式：

**A. 手动 mirror**

```bash
# 安装 coscli (https://cloud.tencent.com/document/product/436/63143)
# 配置 ~/.cos.yaml 后：
for f in ClawHeart_aarch64.dmg ClawHeart_x64.dmg ClawHeart_x64-setup.exe; do
  curl -L -o "/tmp/$f" \
    "https://github.com/tjsdyy/clawheartv2/releases/latest/download/$f"
  coscli cp "/tmp/$f" "cos://<bucket>/clawheart/$f"
done
```

**B. CI 自动 mirror**（未来扩展）

在 `release.yml` 末尾追加一个 `mirror-to-cos` job，需要先在 GitHub repo 配置 secrets：
- `TENCENT_SECRET_ID`
- `TENCENT_SECRET_KEY`
- `COS_BUCKET_NAME`
- `COS_REGION`

然后 job 步骤：

```yaml
mirror-to-cos:
  needs: finalize-aliases
  if: github.event_name == 'push'
  runs-on: ubuntu-latest
  steps:
    - name: Install coscli
      run: |
        wget -q https://github.com/tencentyun/coscli/releases/latest/download/coscli-linux
        chmod +x coscli-linux
    - name: Sync release assets to COS
      env:
        GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      run: |
        for f in ClawHeart_aarch64.dmg ClawHeart_x64.dmg ClawHeart_x64-setup.exe; do
          gh release download "${{ github.ref_name }}" -R ${{ github.repository }} -p "$f" --output "/tmp/$f"
          ./coscli-linux cp "/tmp/$f" "cos://${{ secrets.COS_BUCKET_NAME }}/clawheart/$f" \
            -i "${{ secrets.TENCENT_SECRET_ID }}" -k "${{ secrets.TENCENT_SECRET_KEY }}"
        done
```

镜像就绪后，在 Netlify 项目设置中加 env 覆盖默认 URL：
```
VITE_DESKTOP_DOWNLOAD_MAC_ARM64_URL=https://<bucket>.cos.<region>.myqcloud.com/clawheart/ClawHeart_aarch64.dmg
VITE_DESKTOP_DOWNLOAD_MAC_INTEL_URL=…
VITE_DESKTOP_DOWNLOAD_WIN_URL=…
```

## 故障排查

| 现象 | 原因 / 解决 |
|---|---|
| `macos-x64` job 报 `linker not found` | macos-13 runner 默认 x86_64，正常不应出现；若 runner 升级到 macos-14（ARM），需补 `cargo install cross` |
| `windows-x64` 卡 `webview2` 安装 | tauri-action 默认会下 WebView2 bootstrapper，网络抖动重试即可 |
| `finalize-aliases` 报 `gh: command not found` | ubuntu-latest 默认含 gh CLI；若不在，加 `sudo apt-get install gh` |
| draft Release 没创建 | 检查 push 的 tag 是否匹配 `v*` 模式（必须以 `v` 开头） |
| dmg 在 macOS 提示「来自身份不明的开发者」 | 当前未启用 codesign + notarize；正式发版需配置 Apple Developer 证书 + `APPLE_*` secrets，参考 [tauri-action 文档](https://github.com/tauri-apps/tauri-action#code-signing) |

## 版本号约定

- alpha 阶段：`v2.0.0-alpha.N`（N=0,1,2...）
- 正式 minor：`v2.1.0`
- 补丁：`v2.0.1`

务必保证 `src-tauri/tauri.conf.json::version` 与 tag 名（去掉 `v`）一致 — tauri-action 会用 conf.json 的版本号给产物命名，不匹配会让 finalize-aliases 找不到产物。
