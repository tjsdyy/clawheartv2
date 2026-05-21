# 开发上手（W1 启动周）

本仓库 = ClawHeart Desktop v2 的 Tauri 2 + Rust + React 骨架。

完整 24 周计划见 `readme.md`。

## 先决条件

| 工具 | 版本 | 检查命令 |
|------|------|---------|
| Rust | 1.77+ stable | `rustc --version` |
| Node.js | 20.x+ | `node -v` |
| pnpm | 9.x+ | `pnpm -v` |
| Tauri prereqs | macOS Xcode CLT / Windows MSVC + WebView2 / Linux WebKitGTK+依赖 | [docs.tauri.app](https://v2.tauri.app/start/prerequisites/) |

## 第一次跑

```bash
cd /Users/a1/001.code/clawheartv2

# 安装前端依赖
pnpm install

# Rust 依赖会在第一次 build/dev 时自动拉取
# 启动 dev 模式（前端 + Rust 主进程同时跑）
pnpm tauri:dev
```

首次启动 Rust 依赖编译会比较久（5–10 分钟），后续增量秒级。

## 当前状态（W1 完成度 + 全栈骨架）

### 前端 ✅
- Tauri 2 项目骨架 + 三平台编译就绪
- React + Vite + Tailwind v3 + React Router
- **i18n** (i18next) + zh/en 翻译 + 10 语言架构
- 5 套配色主题（Paper / Carbon / Glacier / Terminal / Cyber）+ localStorage 持久化
- 工具矩阵首页（12 张卡片，11 工具色，扁平 + 彩色）
- **全部 L2 工具页**：监控 / 扫描 / 技能市场 / 安全公告 / 请求日志 / 预算 / 审计 / OpenClaw / Agent / 设置（10 个完整页）
- 3 张 Soon 落地页：Token 验真 / 中转站 / 企业策略
- ⌘K 命令面板（带快捷键、键盘导航）+ 托盘弹窗 + 主题切换器
- 引导页（首次启动 onboarding）

### Rust 内核 ✅ (骨架级)
- **security/** — 12 个文件：normalizer (6 遍) / danger (10 规则) / injection (25 模式) / redact (24 凭据 + Luhn/Mod97) / kill_switch (4 源 OR) / signal / budget / skills / skill_scanner / scanner / advisory / mcp / mcp_chains (10 链) / mcp_baseline
- **proxy/** — 14 个文件：formats / format_detector (9Router 检测) / normalizer trait + 5 协议实现 (openai/claude/gemini/ollama/responses) / usage_extractor / provider_registry / circuit_breaker (4 状态机) / cross_request / streaming / pipeline
- **agents/** — 9 个文件：scanner / process / drift / 6 平台 (claude/codex/cursor/gemini/windsurf/openclaw)
- **storage/** — schema/v2.sql 全表 DDL + 12 query 文件占位 + migrations 程序骨架
- **sync/** — 后台 worker 骨架 + 5 entity 同步占位
- **commands/** — 13 个 IPC 模块（auth/proxy/intercept/skills/danger/budget/agents/scan/advisory/killswitch/status/settings/tools）共 ~40 个命令

### 工程基础 ✅
- `.github/workflows/ci.yml` — fmt+clippy+test 矩阵 3 平台
- `.github/workflows/release.yml` — Tauri action + macOS 公证 + Windows 签名 + 自动更新签名
- `tests/proxy/` — R1 spike 文档 + 24h soak 入口
- `tests/security/` — 注入样本 + 凭据样本 + MCP 攻击链测试 JSON
- `tests/e2e/` — Playwright 场景清单
- `docs/architecture/` + `docs/migration/` + `docs/ui/`

### 🚧 待 W2+ 实施

| 阶段 | 任务 | 涉及模块 |
|------|------|---------|
| W2-W4 | rusqlite 接入 + v1→v2 迁移程序 | Cargo.toml feature `storage` + queries/*::backed |
| W2-W4 | OS Keychain 凭据存储 | keyring crate + commands/auth |
| W4 | tauri-plugin-updater 接入 | tauri.conf.json |
| W5 | **R1 hudsucker 24h spike** | tests/proxy |
| W5-W8 | proxy/server + proxy/tls + 真实 MITM | proxy crate features |
| W9-W12 | Armorer Guard 集成 + 6 遍 NFKC 完整 + 哈希 sha256 | security/mcp + normalizer |
| W14 | i18n 10 语言全填 + lint 强制 | locales/*.json |
| W17 | Agent 真实发现 + fsnotify 漂移 + sysinfo 进程 | agents/* |
| W17 | Ed25519 feed 签名验证 | security/advisory |
| W21 | 第三方安全审计 + 真机 smoke | 全栈 |

## 目录速览

```
clawheartv2/
├── readme.md                 # 项目入口 + 24 周计划
├── DEVELOPING.md             # 本文件
├── package.json              # 前端依赖
├── vite.config.ts            # Vite 配置（端口 1420，给 Tauri）
├── tailwind.config.cjs       # Tailwind v3 + 主题 token 桥接
├── index.html                # SPA 入口（默认带 theme-paper）
├── public/
│   └── clawheart.svg         # logo 占位（SVG）
├── src/                      # React 前端
│   ├── main.tsx
│   ├── App.tsx               # 路由 + 全局快捷键
│   ├── styles/globals.css    # 5 主题 CSS variables + 11 个工具色 token
│   ├── components/
│   │   ├── AppShell.tsx      # header + body + status-bar
│   │   ├── grid/
│   │   │   ├── ToolsGrid.tsx       # L1 工具矩阵首页
│   │   │   ├── ToolCard.tsx        # 单张卡片（扁平 + 彩色）
│   │   │   └── tools.config.ts     # 12 工具配置驱动
│   │   ├── tools/
│   │   │   ├── ToolLayout.tsx      # L2 通用壳（返回按钮 + tabs）
│   │   │   ├── MonitorTool.tsx     # 三栏 master-detail
│   │   │   ├── ScanTool.tsx        # 步骤化扫描
│   │   │   ├── SkillsTool.tsx      # 卡片网格
│   │   │   ├── PlaceholderTool.tsx # 待开发工具占位
│   │   │   └── Onboarding.tsx      # 首次启动引导
│   │   └── overlays/
│   │       ├── CommandPalette.tsx  # ⌘K
│   │       ├── TrayPopup.tsx       # 应用内托盘弹窗（左下）
│   │       └── ThemePicker.tsx     # 主题切换器
│   ├── hooks/
│   │   ├── useTheme.ts        # zustand persist
│   │   ├── useOverlays.ts     # 浮层互斥状态
│   │   └── useOnboarding.ts   # 引导是否完成
│   └── lib/
│       ├── ipc.ts             # Tauri invoke 封装 + 浏览器 fallback mock
│       └── utils.ts           # cn / formatNumber / severityToColor
└── src-tauri/                 # Rust 内核
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── build.rs
    ├── capabilities/default.json
    └── src/
        ├── main.rs
        ├── lib.rs             # Tauri Builder + 注册 invoke_handler
        ├── error.rs           # AppError + thiserror
        ├── state.rs           # AppState（内存 Mutex）
        └── commands/
            ├── mod.rs
            ├── status.rs      # get_status / get_proxy_status
            ├── settings.rs    # get/save_settings / set_theme
            └── tools.rs       # list_tools / list_recent_events / trigger_kill_switch
```

## 可交互的部分

- 首次跑：进入「欢迎使用」引导页 → 点「开始发现」或「跳过」→ 进入工具矩阵
- 顶部角落图标：Agent (徽章 3) / ⌘K 搜索 / Kill Switch / 主题切换 / 设置
- ⌘K (Mac) / Ctrl+K (Win/Linux)：唤起命令面板，方向键导航、Enter 执行
- 状态条左侧"防护中"：点击唤起托盘弹窗（最近事件 + 操作）
- 主题切换 icon (Palette)：5 主题一键切换
- 监控页：点击 stream 行切换右侧详情
- 扫描页：交互的 checkbox + 历史记录
- 技能市场：6 张技能卡（含 disabled / safe / warn / unaudited 4 种状态）

## 重置引导

```js
// 浏览器 devtools console
localStorage.removeItem("clawheart-onboarding");
localStorage.removeItem("clawheart-theme");
location.reload();
```

## 常见问题

**Q: dev 启动报 icon 错？**
A: dev 不需要 icon，但 bundle 需要。见 `src-tauri/icons/README.md`。

**Q: 浏览器直接打开 `http://localhost:1420` 看 UI？**
A: 可以。`lib/ipc.ts` 在非 Tauri 环境会回退 mock 数据，UI 完全可用。

**Q: Tailwind 类不生效？**
A: 确认 `pnpm install` 完成；检查 `tailwind.config.cjs` 中 `content` 是否覆盖到你的文件路径。

**Q: 怎么开始 W2 阶段（接入 SQLite）？**
A: 见 `readme.md` §8 阶段 1 / `src-tauri/Cargo.toml` 里被注释掉的 dependencies + features。

## 下一步建议

1. 跑 `pnpm tauri:dev`，看见工具矩阵
2. 试 5 个配色 + ⌘K + 监控/扫描/技能市场
3. 看一遍 `readme.md` W4 绿灯条件
4. 启动 W5 R1 spike：`tests/proxy/` 下做 hudsucker 24h 真实流量压测
