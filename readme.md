<div align="center">

# ClawHeart Desktop

**本地优先的 AI Agent 安全网关**

让每个 AI 调用可观察 · 可拦截 · 可解释

[![Tauri](https://img.shields.io/badge/Tauri-2.0-FFC131?logo=tauri)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-stable-orange?logo=rust)](https://www.rust-lang.org)
[![React](https://img.shields.io/badge/React-19-61DAFB?logo=react)](https://react.dev)
[![License](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](LICENSE)

[官网](https://clawheart.live) · [下载](https://github.com/tjsdyy/clawheartv2/releases/latest) · [问题反馈](https://github.com/tjsdyy/clawheartv2/issues)

</div>

---

## 是什么

ClawHeart Desktop 是一个**本地常驻**的 AI 安全代理。它接管 Claude Code、Codex、Cursor、OpenClaw、Gemini CLI、Continue、Hermes 等任意 AI Agent 的出站流量，提供拦截、审计、治理和应急能力。

- **零云端依赖**：所有数据停留在本机 SQLite，凭据 100% 入 OS Keychain
- **协议无关**：5 种主流 LLM 协议自动归一化，安全引擎零改动接新协议
- **多档覆盖**：从应用层零侵入到内核级强制隔离，三档监控按需切换
- **失败关闭**：任一安全检查异常默认阻止，绝不放行

## 核心能力

### 三档监控模式

| 档位 | 名称 | 说明 |
|---|---|---|
| **tier1** | 端点映射 | 反向代理监听 `127.0.0.1:19112`。把 Agent 的 `OPENAI_BASE_URL` / `ANTHROPIC_BASE_URL` 指向它即可。无需证书 |
| **tier2** | 系统代理 | hudsucker MITM + 自签 CA，覆盖所有遵循系统代理的进程 |
| **tier3** | 沙箱隔离 | OS 沙箱机制（macOS sandbox-exec / Linux Landlock / Windows AppContainer）强制约束目标进程的网络出口 |

三档可任意组合或单独启用。

### 8 层纵深防御

```
请求入口
  ▼
L0  协议归一化     · 5 种主流协议 → 统一内部表示
  ▼
L1  网络代理 / 熔断 · hudsucker MITM + 流式 SSE 零拷贝
  ▼
L2  内容 DLP       · 48 类凭据指纹 · Luhn / Mod97 / ABA 校验位
  ▼
L3  MCP 攻击链     · JSON-RPC 拦截 + 工具基线冻结 + 漂移检测
  ▼
L4  技能供应链     · SkillGuard 规则引擎（硬触发 + 加权扣分）
  ▼
L5  Agent 漂移     · 配置文件 + memory hash 校验
  ▼
L6  数据 · Token   · 用量统计 + 预算 fail-closed 阻断
  ▼
L7  应急 · KillSwitch · 哨兵不可读自动激活
```

任一层异常 → **失败关闭**（默认阻止 + 返回 LLM 格式错误响应）。

### 主要工具页

| 工具 | 子能力 |
|---|---|
| **监控** | 实时流 · 拦截记录 · Token 用量 · 预算 |
| **扫描** | 80 项本机 AI 安全审计（FilePermission / MCP / Credentials / AgentBehavior / SkillSupplyChain / Sandbox / Network / Windows） |
| **技能备份** | 通配扫描 `~/.<agent>/skills/` · 安全鉴定 · zip 打包 · 备份历史 |
| **Agent** | 自动发现 Claude Code / Codex / Cursor / Continue / OpenClaw 等 · MCP server 列表 |
| **中转配置** | 第三方 LLM API key 集中托管 · Agent 一键覆盖（dry-run + 原子回滚） |
| **设置** | 三档监控切换 · 安全规则自定义 · 实际写入开关 |

### 凭据托管

API key 100% 入 OS Keychain（macOS Security Framework / Linux gnome-keyring / Windows Credential Manager）。Agent 拿到的是 `sk-claw-xxx` 虚拟 key，代理收到请求后用 `credential_router` 反查真实凭据。

数据库、日志、UI 都只见掩码。

## 截图

> 截图待补。可访问 [clawheart.live](https://clawheart.live) 或运行 `pnpm tauri dev` 查看实际效果。

## 安装

### 桌面客户端

到 [Releases](https://github.com/tjsdyy/clawheartv2/releases/latest) 下载对应平台安装包：

- **macOS Apple Silicon**：`ClawHeart_aarch64.dmg`
- **macOS Intel**：`ClawHeart_x64.dmg`
- **Windows x64**：`ClawHeart_x64-setup.exe`

或访问 [clawheart.live](https://clawheart.live) 自动识别平台下载。

### 快速接入

安装并启动后，引导流程会让你选择监控模式（默认推荐 tier1）。然后在你的 Agent 框架或 SDK 中：

```bash
# OpenAI 兼容 SDK
export OPENAI_BASE_URL=http://127.0.0.1:19112/v1

# Anthropic SDK
export ANTHROPIC_BASE_URL=http://127.0.0.1:19112

# 或用 ClawHeart 的「Agent 一键覆盖」向导自动配置所有 Agent
```

### Agent SKILL（让 AI 听懂安全指令）

通过 [`npx skills`](https://skills.sh/) 一行装上 ClawHeart 的 Agent 适配 skill，让 Claude Code / Codex / Cursor / Amp / Cline / OpenClaw 等 12+ Agent 听懂「扫一下 AI 安全」、「鉴定一下技能」、「我装了哪些 Agent」等自然语言，自动调用本机 `clawheart-cli`：

```bash
npx skills add tjsdyy/clawheartv2@clawheart-security -g
```

支持的命令清单见 [`packages/clawheart-skill/clawheart-security/SKILL.md`](./packages/clawheart-skill/clawheart-security/SKILL.md)。

> 前置：`clawheart-cli` 需在 PATH。每个 Release 都附带独立的 CLI tarball（mac arm64 / mac x64 / windows x64），见 [Release 页](https://github.com/tjsdyy/clawheartv2/releases/latest) 里 `clawheart-cli-*.tar.gz` / `clawheart-cli-*.zip`。

## 从源码运行

### 工具链

```bash
# Rust（stable）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node 20+ 和 pnpm
brew install node           # macOS
npm install -g pnpm

# Tauri CLI
cargo install tauri-cli
```

### 启动开发模式

```bash
git clone https://github.com/tjsdyy/clawheartv2.git
cd clawheartv2
pnpm install
pnpm tauri dev
```

### 构建

```bash
pnpm tauri build                                # 当前平台
pnpm tauri build --target aarch64-apple-darwin  # 指定 target
```

### 测试

```bash
cargo test --workspace  # Rust 单测
pnpm typecheck          # 前端类型检查
```

## 项目结构

```
clawheartv2/
├── src-tauri/                  # Rust 内核
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/           # IPC 权限白名单
│   ├── schema/v2.sql           # SQLite schema
│   └── src/
│       ├── commands/           # IPC 命令层
│       ├── proxy/              # 代理引擎（hudsucker + 协议归一化）
│       ├── security/           # 安全引擎（DLP / MCP / 注入 / 80 项扫描）
│       ├── agents/             # Agent 发现 + 平台探针
│       ├── skills/             # 技能发现 + 备份 + SkillGuard 扫描
│       ├── storage/            # rusqlite + 迁移
│       └── sync/               # 云同步骨架
├── src/                        # React 前端
│   ├── components/
│   │   ├── grid/               # 工具矩阵首页
│   │   ├── tools/              # 各工具 L2 页面
│   │   ├── access-mode/        # 三档监控模式
│   │   ├── providers/          # 中转 API 管理
│   │   └── overlays/           # 命令面板 / 主题选择
│   ├── hooks/                  # TanStack Query hooks
│   ├── lib/                    # IPC 封装 + utils
│   └── locales/                # i18n（10 种语言）
├── .github/workflows/          # CI：三平台矩阵编译 + Release 自动化
└── docs/                       # 文档
```

## 技术栈

| 层 | 选型 | 选型理由 |
|---|---|---|
| 桌面壳 | **Tauri 2** | 单二进制 + 系统 WebView，包体小、内存低 |
| 内核语言 | **Rust** | 内存安全、零拷贝、cargo 工具链统一 |
| 代理引擎 | **hudsucker 0.22** | Rust 生态成熟的 HTTPS MITM |
| TLS / 证书 | **rustls + rcgen** | 纯 Rust，跨平台一致 |
| 异步运行时 | **tokio 1.x** | 标准事实 |
| 存储 | **rusqlite (bundled)** | 零原生依赖，跨平台编译无痛 |
| 凭据存储 | **OS Keychain** (`keyring` crate) | 三平台统一 API |
| 前端框架 | **React 19 + TypeScript** | 主流，类型安全 |
| 样式 | **Tailwind CSS 3** | 零内联 |
| 数据加载 | **TanStack Query v5** | 取代手动 fetch + setState |
| 图标 | **Lucide React** | 单色矢量 |
| MCP 安全 | **自实现 JSON-RPC 拦截器** | `src-tauri/src/proxy/mcp/` |
| 公告签名 | **Ed25519** (`ed25519-dalek`) | 标准、轻量 |
| 自动更新 | **tauri-plugin-updater** | minisign 公钥校验 |

## 安全自治原则

> 守护别人的工具，自身必须更干净。

- **依赖最小化**：Rust 内核零 npm 依赖；前端 dep 精简
- **CI 守护**：`cargo-audit` + `cargo-deny` 每 PR 跑
- **二进制签名**：macOS 公证 + Windows EV 证书 + Linux minisign
- **Tauri Capabilities**：每个 IPC 命令显式白名单
- **失败关闭**：任一安全检查 panic / timeout → 走 block 路径
- **不出网默认**：主进程零网络出口；只代理 + sync worker 可出网且 URL 白名单
- **审计日志**：所有拦截事件附 MITRE ATT&CK ID

## 贡献

欢迎贡献！请遵循：

1. Fork 仓库并开新分支
2. 跑通 `cargo test --workspace` + `pnpm typecheck`
3. 提交 PR，描述清楚改动动机
4. PR 会通过 CI 三平台编译验证

新的检测规则、Agent 探针、协议归一化适配尤其欢迎。

## License

[Apache License 2.0](LICENSE)
