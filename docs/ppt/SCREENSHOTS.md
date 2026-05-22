# 截图清单 · ClawHeart 新手操作指南 PPT

把截图保存到 `./screenshots/` 目录，**严格按下面的文件名命名**（HTML PPT 写死了引用路径）。

格式：PNG，建议宽度 ≥ 1600px（4K Retina 屏直接截一次窗口即可）。

---

## 章节 1 · 安装（4 张）

| 文件名 | 截什么 | 操作 |
|--------|--------|------|
| `01-install-dmg.png` | DMG 文件双击打开后的窗口（左侧 app icon + 右侧 Applications 快捷方式） | Finder 双击 DMG |
| `02-drag-to-applications.png` | 拖动 ClawHeart.app 到 Applications 文件夹的过程 | DMG 窗口截图，含拖动光标 |
| `03-launch-warning.png` | macOS 首次打开提示「无法验证开发者」对话框 | 首次双击 App，截弹窗 |
| `04-allow-in-settings.png` | 系统设置 → 隐私与安全性 → "仍要打开" 按钮 | 设置页面 |

---

## 章节 2 · 首次启动（3 张）

| 文件名 | 截什么 |
|--------|--------|
| `05-onboarding-welcome.png` | 引导页第一屏 |
| `06-onboarding-tier.png` | 引导页中的监控模式选择步骤 |
| `07-home-hero.png` | 主页（含 Hero 动画 + 底栏 7 个工具卡） |

---

## 章节 3 · 发现本机 Agent（3 张）

| 文件名 | 截什么 |
|--------|--------|
| `08-agents-page.png` | Agent 发现页（顶部多个 Agent tab，注意右上"可导入"角标） |
| `09-agents-candidate.png` | 候选 Agent 区（"在 ~/.xxx/ 发现 AI 线索"折叠区展开） |
| `10-agents-confirm.png` | 点击"纳入管理"后的状态 |

---

## 章节 4 · 模型渠道管理（5 张）

| 文件名 | 截什么 |
|--------|--------|
| `11-providers-empty.png` | 渠道管理页首次进入（空状态 + 新建按钮） |
| `12-add-channel-presets.png` | 新增渠道弹层，展示 65 个预设网格（OpenAI / Anthropic / DeepSeek / Kimi 等） |
| `13-add-channel-form.png` | 选中预设后表单（Base URL 已自动填，要填 API Key） |
| `14-providers-list.png` | 已创建几个渠道后的列表（每条显示分配给哪些 Agent） |
| `15-manage-assignments.png` | 「管理分配」弹层（勾选要分配给哪些 Agent） |

---

## 章节 5 · 从配置自动导入（4 张，OpenClaw 用户特有）

| 文件名 | 截什么 |
|--------|--------|
| `16-import-button.png` | Agent tab 顶部的 [📥 从配置导入] 按钮（仅 OpenClaw/OpenEva/Codex 等显示） |
| `17-import-dialog-candidates.png` | 导入弹层 + 候选列表（含 OAuth warning） |
| `18-import-selected.png` | 勾选要导入的候选 + 底部 [导入 N] 按钮 |
| `19-after-import.png` | 导入完成后渠道列表新增的条目 |

---

## 章节 6 · 启用托管（5 张）

| 文件名 | 截什么 |
|--------|--------|
| `20-channel-row.png` | 已分配渠道的列表行（含「启用」按钮） |
| `21-wizard-step1.png` | OverwriteWizard step 1 选渠道 |
| `22-wizard-step3-diff.png` | step 3 配置文件 diff 预览 |
| `23-takeover-active.png` | 工具栏「已托管」开关 + 当前渠道显示 |
| `24-tab-status-dot.png` | Agent tab 末尾绿色状态点（已托管） |

---

## 章节 7 · 监控与回滚（3 张）

| 文件名 | 截什么 |
|--------|--------|
| `25-history-drawer.png` | 「📜 托管历史」抽屉，列出所有变更 |
| `26-rollback-confirm.png` | 单条回滚的确认弹窗 |
| `27-monitor-realtime.png` | 实时监控页（流量 / 拦截记录） |

---

## 章节 8 · 设置 & 多语言（2 张）

| 文件名 | 截什么 |
|--------|--------|
| `28-settings-language.png` | 设置页 → 通用 → 语言下拉，10 种语言列出 |
| `29-en-ui.png` | 切换到英文后的主页（演示多语言生效） |

---

## 章节 9 · 进阶：监控模式（2 张）

| 文件名 | 截什么 |
|--------|--------|
| `30-access-modes.png` | 监控模式页（3 张 Tier 卡片对比） |
| `31-tier1-detail.png` | Tier 1 端点映射的配置面板（展开后的详情 + 协议清单） |

---

## 截图小贴士

- **Mac 全屏窗口**：用 `Cmd+Shift+5` → 选「截取选定窗口」，自动带阴影背景，最专业
- **马赛克敏感信息**：API key / virtual_key 前缀保留 `sk-claw-`、后缀**模糊化**或用 `••••` 替代
- **保留 UI 状态**：能展开的菜单/弹层尽量展开，让一张图传达更多信息
- **缩放比例统一**：所有截图建议在同一 ClawHeart 窗口大小下拍（如 1280×800），便于 PPT 视觉一致

完成全部 31 张后，直接打开 `index.html` 即可看到完整 PPT。缺图的页面会自动显示占位提示。
