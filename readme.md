# ClawHeart Desktop v2

> 本机 AI 安全运行时 · Tauri 2 + Rust · 24 周到 GA
>
> 版本：1.0 · 启动日期：2026-05-17 · 目标 GA：2026-11-01
>
> 阅读对象：工程团队 + 产品决策者 · 本文件 = 项目入口 + 24 周开发计划

---

## 0. 一页摘要

把现在的 **Electron + Express + axios** 桌面客户端，重写为以 **Tauri 2 + Rust** 为内核的本机 AI 安全代理客户端。承担「**拦截 / 审计 / 治理 / 应急**」四项职责，覆盖 Claude Code、Codex、Gemini CLI、Cursor、OpenClaw 等任意 AI Agent 的出站流量。

**4 条核心差异化**（按优先级）：

/

| \# | 差异化 | 一句话 | 借鉴 |
| --- | --- | --- | --- |
| 1 | 协议无关化 | 5 种 LLM 协议自动归一化，安全引擎零改动接新协议 | 9Router |
| 2 | MCP 深度安全 | JSON-RPC 工具扫描 + 攻击链子序列 + 工具基线冻结 | Pipelock |
| 3 | 技能供应链信任链 | 72 条规则预扫描 + Ed25519 签名公告 + 漂移检测 | SkillGuard + ClawSec |
| 4 | 失败关闭哲学 | 所有错误路径默认阻止，4 源 OR Kill Switch | Pipelock |

**性能目标（vs v1）**：

| 维度 | v1 | v2 目标 |
| --- | --- | --- |
| 安装包 | 200MB+ | **\<15MB** |
| 空闲内存 | 150–300MB | **\<40MB** |
| 启动时间 | 3–5s | **\<1s** |
| 代理延迟 | \~50ms（同步缓冲） | **\<5ms**（零拷贝流式） |
| 协议覆盖 | 2 种 | **5 种** |
| MCP 安全 | 0% | **OWASP MCP Top 10 + 10 条攻击链** |
| 防御层 | 3 | **8** |

---

## 1. 项目状态

```text
W0 [当前] ───── W4 ───── W8 ───── W12 ───── W16 ───── W20 ───── W24 GA
  ●            ●         ●         ●         ●          ●         ◆
启动准备      基座     代理核心   安全引擎    UI完成     高级能力   打磨发布
```

- **当前阶段**：W0 启动准备
- **下一里程碑**：W4 基座绿灯（Tauri 骨架 + 迁移程序 + 系统托盘）
- **GA 目标**：2026-11-01

---

## 2. 文档与产出物索引

| 文档 | 路径 | 作用 |
| --- | --- | --- |
| **架构蓝图** | `../opencarapace-server/local-desktop/CLAWHEART_V2_BLUEPRINT_CN.md` | 工程团队执行版（模块 / trait / schema 粒度） |
| **UI 工具箱方案** | `../opencarapace-server/local-desktop/.context/ui-mocks-v2.md` | 工具箱范式 + 2 层深度 + 5 个工具页 mock |
| **HTML 原型** | `../opencarapace-server/local-desktop/.context/ui-prototype-v2.html` | 单文件高保真，5 配色 + 5 视图可交互 |
| **整合日志** | `../opencarapace-server/local-desktop/.context/note.md` | 决策来源、争议点、后续待办 |
| **本文件** | `readme.md` | 项目入口 + 24 周开发计划 |
| 旧版 5 方案 mock | `../opencarapace-server/local-desktop/.context/ui-mocks.md` | 已被工具箱方案取代，留档 |

**6 份输入材料**（位于 `../opencarapace-server/local-desktop/` 旁）：9Router / Pipelock / SkillGuard / OpenClaw Audit / ClawSec / CC-Switch 竞品分析。

---

## 3. 产品定位（边界明确）

**我们是**：

- 本机常驻的 AI 安全代理 + 审计 UI + 治理控制台
- 协议无关的内联扫描器（流量层 + MCP JSON-RPC 层）
- 用户对自家所有 AI Agent 行使主权的入口（停、限、拦、看）

**我们不是**：

- 不是 LLM 路由器（不抢 9Router/CC-Switch 心智，不做 N×N 翻译）
- 不是终端审计工具（不抢 OpenClaw Audit 的离线扫描心智，但收编它的 80 项检查）
- 不是 OpenClaw 配置管理器（OpenClaw 集成是「能力」非「主线」）

---

## 4. 技术栈与选型

| 层 | 选型 | 关键理由 |
| --- | --- | --- |
| 桌面壳 | **Tauri 2** | 单二进制 + 系统 WebView，包体 / 内存 / 启动皆达标 |
| 内核语言 | **Rust** | 内存安全、零拷贝、cargo 工具链统一 |
| 代理引擎 | **hudsucker** | Rust 生态唯一成熟 HTTPS MITM；R1 需在 W5 做 24h 压测验证 |
| TLS / 证书 | **rustls + rcgen** | 纯 Rust，跨平台一致 |
| 异步运行时 | **tokio 1.x** | 标准事实 |
| 存储 | **rusqlite \(bundled\)** | 零原生依赖，跨平台编译无 MSVC/Xcode 困扰 |
| 凭据存储 | **OS Keychain** \(`keyring` crate\) | 不再 SQLite 明文 |
| 前端框架 | **React + TypeScript** | v1 沿用，降低风险 |
| 样式 | **Tailwind v4** | 零内联（v1 痛点） |
| 组件 | **Radix Primitives + shadcn/ui**（复制式） | 借鉴 CC-Switch |
| 数据加载 | **TanStack Query v5** | 取代 v1 手动 fetch + setState |
| 图标 | **Lucide React** | 单色矢量 |
| 表单 | **react-hook-form + zod** | 类型安全 |
| 图表 | **uPlot** | 远轻于 Recharts，符合 \<40MB 内存目标 |
| MCP 安全 | **Armorer Guard**（外部 crate） | R2 待 W9 验证，准备 fallback |
| 公告签名 | **Ed25519** \(`ed25519-dalek`\) | 标准、轻量 |
| 自动更新 | **tauri-plugin-updater** | minisign 公钥校验 |
| i18n | **i18next** + type-safe key | 保留 10 种语言 |

**禁用清单**（避免重蹈 v1）：

- ❌ Electron / Node 二进制 / 内嵌 SDK 工具链
- ❌ axios（同步缓冲整段响应）
- ❌ 内联 style 属性（lint 强制）
- ❌ CORS `*`（用 Tauri Capabilities 显式授权 IPC）

---

## 5. 目录结构（开仓即按此布局）

```text
clawheartv2/
├── readme.md                    # 本文件
├── src-tauri/                   # Rust 内核
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── main-window.json     # IPC 权限白名单
│   └── src/
│       ├── main.rs              # Tauri 入口（含 Linux WebKit workaround）
│       ├── lib.rs               # 模块聚合
│       ├── error.rs             # AppError + thiserror
│       ├── state.rs             # AppState（共享句柄）
│       ├── commands/            # IPC 命令层（薄）
│       ├── proxy/               # 代理引擎（hudsucker + 归一化）
│       ├── security/            # 安全引擎（DLP / MCP / Kill Switch）
│       ├── agents/              # Agent 发现 + 漂移监控
│       ├── sync/                # 云端同步 worker
│       └── storage/             # rusqlite + 迁移
├── src/                         # 前端
│   ├── main.tsx
│   ├── App.tsx                  # 路由：/ 与 /tools/:toolId
│   ├── components/
│   │   ├── ui/                  # shadcn 复制
│   │   ├── grid/                # 工具矩阵
│   │   └── tools/               # 各工具 L2 页
│   ├── hooks/                   # useTools / useBadges / useEventStream
│   ├── lib/                     # ipc 封装 / utils
│   └── locales/                 # 10 种语言 JSON
├── docs/
│   ├── architecture/            # 架构蓝图（拷贝）
│   ├── ui/                      # mocks + 原型（拷贝）
│   └── migration/               # v1→v2 迁移指南
├── tests/
│   ├── e2e/                     # Playwright + Tauri WebDriver
│   ├── proxy/                   # hudsucker 24h soak
│   └── security/                # 攻击链 / DLP / 注入样本集
└── .github/
    └── workflows/               # 矩阵编译 + 公证 + 签名
```

详细模块清单见架构蓝图 §4。

---

## 6. 8 层防御（v1 是 3 层）

```text
请求入口
  ▼
L0  格式归一化（5 种协议 → NormalizedRequest）
  ▼
L1  网络层（hudsucker MITM + 流式 SSE + 熔断）
  ▼
L2  内容层（DLP 先于 DNS + 6 遍归一化 + 48 凭据模式）
  ▼
L3  协议层（MCP JSON-RPC 扫描 + 攻击链 + 基线冻结）
  ▼
L4  供应链层（72 规则预扫描 + Ed25519 公告）
  ▼
L5  系统层(Agent 进程发现 + 文件漂移)
  ▼
L6  数据层（Keychain + 类保留脱敏 + MITRE ATT&CK 映射）
  ▼
L7  应急层（Kill Switch 4 源 OR · 失败关闭）
```

**全局不变量**：任一层抛错 → **失败关闭**（默认阻止 + 返回 LLM 格式 block 响应）。

---

## 7. UI 范式（已定稿）

**工具箱 + 2 层深度**：

- **L1 = 工具矩阵首页**：4×N 卡片网格，居中悬浮（不撑满），每张卡片 \~163px 正方形
- **L2 = 单工具内部**：用 Tab 平铺，**绝不嵌新页**
- **角落 icon 最多 4 个**：👥 Agent · 🔍 ⌘K · 🚨 Kill Switch · ⚙ 设置
- **事件感知三件套**：托盘弹窗 + 系统通知 + 卡片徽章

**默认配色**：Paper（羊皮纸浅色）；提供 Carbon / Glacier / Terminal / Cyber / Paper 5 套主题。

**前端路由**（极简）：

```text
"/"                          → L1 工具矩阵
"/tools/:toolId"             → L2 工具内部
"/tools/:toolId?tab=:tabId"  → L2 Tab 切换（不算新路由）
```

**强制约束**（lint 规则）：

- 禁止 `tools/*` 子组件跳到 `/tools/*` 之外路径
- 禁止 L2 子组件嵌 `<Routes>` / `<Router>`
- 矩阵 `tools[]` 配置变更必须过 PR review

详见 UI 工具箱方案 + 原型。

---

## 8. 24 周路线图（6 阶段 × 4 周）

> 每阶段都有"绿灯条件"，**不达标不进入下一阶段**。

### 阶段 1 \(W1–W4\) — 基座

**做什么**：

- Tauri 2 项目骨架（macOS / Windows / Linux 三平台编译通过）
- rusqlite + schema\_migrations 表 + v1→v2 迁移程序
- 设置 / 认证 IPC 命令（含 Keychain 写入）
- 工具矩阵首页骨架（按工具箱范式 1:1 静态化）
- 系统托盘空壳（绿/黄/红三色图标）
- 登录 / 登出 / 刷新 token（OS Keychain）

**绿灯条件**：

- `cargo run` + `pnpm tauri dev` 起得来；3 平台都能编译
- v1→v2 迁移程序在 3 套真实 v1 数据上跑通（含拆密钥到 Keychain）
- 登录/登出/刷新完整，token 落 Keychain
- 系统托盘可见，左键弹 mini UI，右键菜单可用
- macOS arm64 包尺寸 \< 15MB

**关键风险**：R5（CA UX）需要在本阶段同步设计稿。

---

### 阶段 2 \(W5–W8\) — 代理核心

**做什么**：

- hudsucker 起转 + CA 自动颁发 + 三平台系统信任脚本
- L0–L1 全量实现（格式检测 + OpenAI/Claude 归一化）
- 流式 SSE 真透传（逐字、零拷贝）
- 请求日志 100% 落库
- 预算 + 危险指令（v1 等价行为）
- 监控 L2 页面（实时流 + 拦截记录 Tab）

**绿灯条件**：

- 19111 通过证书可拦 Claude Code / OpenAI SDK 流量
- SSE 流式响应**逐字**透传到 SDK，端到端延迟 \< 上游 + 5ms
- v1 的危险指令 / 技能禁用拦截**行为等价**（相同样本，命中/通过结果一致）
- 请求日志 100% 落库；UI 监控面板可看
- 1k QPS 压测 30 分钟，内存稳定不漏

**关键风险**：**R1 必须在 W5 第一周完成 spike**（hudsucker 24h 真实流量压测）。如失败，回退到自研 `hyper + rustls`，需追加 4 周缓冲。

---

### 阶段 3 \(W9–W12\) — 安全引擎

**做什么**：

- 6 遍归一化（zero-width / 同形字 / Leetspeak / base64 / NFKC / whitespace）
- 注入检测 25 模式
- MCP JSON-RPC 拦截器（HTTP/SSE 模式优先；stdio 后置 v2.1）
- MCP 攻击链 10 条 pattern + 间隙容忍子序列匹配
- 工具基线冻结 + 描述漂移检测
- 48 凭据 DLP + Luhn/Mod97/ABA 校验位
- Armorer Guard 集成（含 fallback 路径）
- 信号分类（Threat / Protective / ConfigMismatch / InfraError）
- Kill Switch 4 源

**绿灯条件**：

- 100 条注入样本（含零宽/同形字/Leetspeak/base64）召回率 ≥ 90%、误报率 ≤ 5%
- MCP 攻击链 10 条 pattern 合成会话 100% 命中；间隙容忍 5 次混淆调用不丢
- Kill Switch 4 源单测 + e2e；哨兵文件被 chmod 000 → 5 秒内激活
- DLP 48 条全覆盖单测，校验位减误报达标
- 所有拦截事件带 MITRE ATT&CK ID

**关键风险**：**R2 必须在 W9 前完成 Armorer Guard 决断**（沟通 + 基准）。备选：自带轻量内核 + 规则库 fallback。

---

### 阶段 4 \(W13–W16\) — UI 完成度

**做什么**：

- 工具矩阵 + 9 个 L2 工具页面（监控 / 扫描 / 技能市场 / 安全公告 / 请求日志 / 预算 / 审计报告 / OpenClaw / Agent）按原型 1:1 实现
- 5 配色主题动态切换（持久化到 settings）
- TanStack Query v5 接所有 IPC
- i18n 10 种语言迁移（type-safe key）
- ⌘K 命令面板 + 系统托盘弹窗 + 紧凑模式
- 引导流程（Agent 发现 onboarding）
- 3 种窗口尺寸适配（紧凑 / 标准 / 宽屏）

**绿灯条件**：

- App.tsx 35KB 内联样式归零（lint 强制）
- 国际化覆盖率 100%（缺 key CI 失败）
- 首屏 \< 1s，TTI \< 1.5s（系统 WebView）
- 设计走查：拦截事件卡片、技能详情、Agent 树三个核心场景产品验收
- 5 配色在 3 平台 × 暗/亮 OS 模式下视觉走查通过

**关键风险**：R4（Linux WebView 一致性）需在 Ubuntu / Fedora / Arch 各跑一遍核心面板。

---

### 阶段 5 \(W17–W20\) — 高级能力

**做什么**：

- Agent 发现（6 平台：claude / codex / gemini / cursor / windsurf / openclaw）
- 进程扫描 + 配置目录监控
- MCP server 枚举
- 文件漂移监控（fsnotify + sha256 基线）
- 技能扫描器（SkillGuard 72 条规则）
- 安全公告订阅（Ed25519 feed + 漂移守护）
- 增量云同步
- tauri-plugin-updater（alpha / beta / rc 三档信道）

**绿灯条件**：

- 装有 Claude Code / Codex / Cursor / Gemini CLI 的真实机器自动列出 ≥ 80% Agent + 配置
- 漂移检测：篡改 `~/.claude/CLAUDE.md` → 5 秒内告警 + 一键还原
- 技能扫描器跑 SkillGuard 公开样本集，与官方结果差异 ≤ 2%
- 公告 feed 签名验证：被篡改 feed 必拒；漂移守护警报必触
- 自动更新 alpha → beta → rc 三档信道走通

---

### 阶段 6 \(W21–W24\) — 打磨与发布

**做什么**：

- 三平台真机 smoke test
- 第三方安全审计
- 文档：用户手册 + 开发者文档 + v1→v2 迁移指南
- Beta 公测（≥ 50 用户、14 天）
- 发布矩阵：macOS arm64 / macOS x64 / Windows x64 / Linux x64 deb+AppImage
- GA

**绿灯条件**：

- 三平台真机 smoke test 全通过
- 外部安全 review 报告 Critical / High 数 = 0
- ≥ 50 名 Beta 用户在 14 天内零关键 issue
- release notes + 升级指南 + FAQ 上线
- macOS 公证 + Windows EV 签名 + Linux minisign 全部走通

---

## 9. W1 启动周计划（细到天）

> 第一周不能只是"开会"。下方任务可分配到具体人。

| 天 | 任务 | 产出 |
| --- | --- | --- |
| **Mon \(D1\)** | 启动 kickoff（架构同步 + 角色分工） · 仓库初始化（`cargo new --bin` + `pnpm create tauri-app`） · `.github/workflows/ci.yml` 三平台 matrix | 仓库可 push · CI 跑通 |
| **Tue \(D2\)** | Tauri 2 骨架最小化跑通 · `tauri.conf.json` + `capabilities/main-window.json` 模板 · 接入 Tailwind v4 + Radix + shadcn 复制 | 空窗口可启动 |
| **Wed \(D3\)** | rusqlite 接入 · `storage/migrations.rs` 框架 · v2 schema 全量 DDL 落地（参考蓝图 §6） · 编写 `schema_migrations` 自举逻辑 | 启动即建库 |
| **Thu \(D4\)** | v1→v2 迁移程序骨架 · 真实 v1 数据库样本 3 套（自己 / 同事 / clean-install）入库测试 | 迁移程序 dry-run 通过 |
| **Fri \(D5\)** | 系统托盘空壳 + 三色图标资源 · 工具矩阵 React 骨架（按原型 1:1 静态化） · **R1 spike 启动**：hudsucker hello world + 真实 Claude Code 24h 后台压测开跑 | 托盘可见 · 矩阵可见 · spike 跑起 |

**周五 demo**：跑通 `pnpm tauri dev`、看见工具矩阵首页、托盘可见、迁移 dry-run 报告输出。

---

## 10. 关键风险与开放问题（开工前必须知道）

| \# | 风险 | 影响阶段 | 决策窗口 | 备选 |
| --- | --- | --- | --- | --- |
| **R1** | hudsucker 1k QPS × 24h 稳定性未验证 | 阶段 2 起 | W5 第一周 spike | 回退自研 `hyper + rustls`（参考 mitmproxy\_rs） |
| **R2** | Armorer Guard 可用性、活跃度、性能 | 阶段 3 | W9 前 | 抽象接口 + 自带轻量内核 + 规则库 fallback |
| **R3** | MCP stdio 模式如何拦截 | 阶段 3 + v2.1 | v2.0 文档化"stdio = best-effort baseline" | v2.1 调研 fd 注入 / wrap binary / proxy bridge |
| **R4** | Linux WebView 各发行版漂移 | 阶段 4 | W13 | 关键功能降级（如禁动画） |
| **R5** | CA 信任 UX 摩擦（macOS / Windows / Linux 异） | 阶段 2 | W5 | 引导流程必走完 + 视频指南；退路：不装 CA 降级到正向代理 |
| **R6** | OpenClaw 路径降级老用户感知 | 阶段 5 + 发布 | 发布前 | release notes 明确：仍支持，但不再主导 |
| **R7** | 企业版/团队版规划 | v2.3 | v2.0 不做 | schema 预留 `org_id / policy_version` 字段 |

---

## 11. 安全自治原则（产品自身的保证）

> 守护别人的工具，自身必须比别人都干净。

| 维度 | 要求 |
| --- | --- |
| 依赖最小化 | Rust 内核零 npm 依赖；前端 dep \< 20 个；每个依赖入选给理由 |
| 核心审计 | `proxy / security / storage` 强制 review；外部 PR 不直接 merge |
| CI 守护 | cargo-audit + cargo-deny + trivy + codeql 每 PR 跑 |
| 二进制签名 | macOS 公证 + Windows EV 证书 + Linux minisign |
| Tauri Capabilities | 每个 IPC 命令显式白名单；遗漏即拒绝 |
| 失败关闭 | 任一安全检查 panic / timeout → 走 block 路径 |
| 不出网默认 | 主进程零网络出口；只代理 + sync worker 可出网，且 URL 白名单 |
| 审计日志 | 所有拦截事件附 MITRE ATT&CK ID（T1059 / T1048 / T1195.002） |
| 应急预案 | 4 源 Kill Switch；哨兵不可读 = 激活（失败关闭） |
| CA 私钥 | OS Keychain / DPAPI 加密；退出清理内存映射 |
| CVE 响应 | dependabot + osv-scanner + watch list；P1 漏洞 SLA 72h |

---

## 12. 14 类核心威胁清单（每条都有 v2 应对）

| \# | 威胁 | v2 应对 | 模块 |
| --- | --- | --- | --- |
| T1 | 危险指令 | 正则 + 6 遍归一化 + 上下文 | `security/danger.rs` + `injection.rs` |
| T2 | 提示词注入 | 25 模式 + 同形字 + Leetspeak | `security/injection.rs` |
| T3 | 被禁/恶意技能调用 | 黑名单 + Armorer 内联 | `security/skills.rs` |
| T4 | 恶意技能包安装 | 72 条预扫描 + 评分 | `security/skill_scanner.rs` |
| T5 | 技能供应链 | SK-005 / SK-008 复用 | `security/skill_scanner.rs` |
| T6 | MCP 提示词注入 | Armorer 扫描 JSON-RPC | `security/mcp.rs` |
| T7 | MCP 工具投毒 | 基线冻结 + 哈希比对 | `security/mcp_baseline.rs` |
| T8 | MCP 攻击链 | 10 种序列 + 间隙容忍 | `security/mcp_chains.rs` |
| T9 | 凭据外泄 | 48 模式 + 校验位 + 类保留脱敏 | `security/redact.rs` |
| T10 | 跨请求分片外泄 | 域内字节预算 + 分片重组 | `proxy/cross_request.rs` |
| T11 | 预算超限 | provider × model × period | `security/budget.rs` |
| T12 | 流氓 Agent | 进程 + 配置文件扫描 | `agents/scanner.rs` |
| T13 | CVE / 公告未升级 | Ed25519 feed + 已装匹配 | `security/advisory.rs` |
| T14 | 客户端自身被滥用 | Tauri IPC（架构层面消除 CORS） | 架构层面 |

---

## 13. 验收门禁（每阶段必过）

```text
┌──────────┬─────────────────────────────────────────┐
│ 阶段     │ 必过门禁                                  │
├──────────┼─────────────────────────────────────────┤
│ W4 基座  │ 包 <15MB · 三平台编译 · 迁移 dry-run 通过 │
│ W8 代理  │ 1k QPS × 30min · SSE 逐字 · v1 行为等价  │
│ W12 安全 │ 注入召回≥90% · MCP 链 100% · KS 5s 激活   │
│ W16 UI   │ 内联样式=0 · i18n 100% · 首屏<1s         │
│ W20 高级 │ Agent 发现≥80% · 漂移 5s · feed 验签必过  │
│ W24 GA   │ 外审 0 Critical/High · 50 Beta 零关键 14d │
└──────────┴─────────────────────────────────────────┘
```

---

## 14. 如何开始

```bash
# 1. 准备工具链
rustup install stable
rustup target add aarch64-apple-darwin x86_64-apple-darwin x86_64-pc-windows-msvc x86_64-unknown-linux-gnu
cargo install tauri-cli
pnpm install -g pnpm

# 2. 克隆与初始化
git clone <repo> clawheartv2
cd clawheartv2
pnpm install
(cd src-tauri && cargo fetch)

# 3. 开发
pnpm tauri dev          # 启动开发模式
cargo test --workspace  # Rust 测试
pnpm test               # 前端测试

# 4. 构建发布
pnpm tauri build        # 当前平台
```

**首次 onboarding 检查清单**：

- 读完本文件
- 读架构蓝图（重点 §3 / §4 / §5 / §11）
- 浏览 UI 原型（5 配色 + 5 视图）
- 在本机跑通 v1 客户端（建立基线认知）
- 加入 R1 / R2 spike 讨论组

---

## 15. v1 → v2 端到端 diff（核心 18 项）

| 维度 | v1 | v2 |
| --- | --- | --- |
| 运行时 | Electron + Node | Tauri 2 + Rust 单二进制 |
| 代理 | Express + axios 同步缓冲 | hudsucker MITM 零拷贝流式 |
| 协议 | OpenAI Chat | 5 种自动识别 |
| 安全 | 子串匹配 | 正则 + 归一化 + MCP + DLP + 攻击链 |
| MCP | ❌ | ✅ JSON-RPC + 攻击链 + 基线 |
| 技能扫描 | ❌ | 72 条预扫描 + 评分阻断 |
| 公告 | ❌ | Ed25519 签名 feed |
| Agent 发现 | 演示数据 | 进程 + 配置文件扫描 6 平台 |
| Kill Switch | ❌ | 4 源 OR |
| 凭据 | SQLite 明文 | OS Keychain |
| UI | App.tsx 35KB 内联 | Tailwind v4 + Radix + shadcn |
| 数据加载 | 手动 fetch | TanStack Query v5 |
| 系统托盘 | ❌ | 三色 + mini UI |
| 自动更新 | ❌ | tauri-plugin-updater |
| 平台 | macOS / Windows | + Linux |
| 包尺寸 | 200MB+ | \<15MB |
| 内存 | 150–300MB | \<40MB |
| 启动 | 3–5s | \<1s |

---

## 16. 不在 v2.0 范围（明确切出）

- **MCP stdio MITM**：v2.0 只做 best-effort baseline + 配置检查，stdio 真正拦截推到 v2.1
- **企业版 / 团队版 / 集中策略下发**：v2.3 路线；但 schema 预留 `org_id / policy_version`
- **多供应商一键切换 + 故障转移**：明确不做，避免抢 9Router / CC-Switch 心智
- **OpenClaw 全功能内置**：默认 Core-first，OpenClaw 仅作为可选集成
- **Token 中转站验真 / 中转站功能**：UI 已留 Soon 卡，落地排 v2.1

---

## 17. 责任与决策

| 角色 | 职责 |
| --- | --- |
| 架构组 | 蓝图、R1/R2/R3 决策、模块边界 |
| Rust 工程 | `proxy / security / agents / sync / storage` |
| 前端工程 | 工具矩阵 + 9 个 L2 工具页 + i18n |
| 安全 | DLP / 攻击链 / Kill Switch / 外审协调 |
| 设计 | UI 原型迭代 + 5 配色定稿 + 引导流程 |
| 发布 | CI 矩阵 + 公证 + 签名 + 自动更新通道 |

---

## 附录 — 30 借鉴点 P0/P1/P2 一览（与模块映射）

详见架构蓝图 附录 A。摘 P0 列表：

| \# | 借鉴 | 来源 | 落点 |
| --- | --- | --- | --- |
| 1 | 协议格式归一化 | 9Router | `proxy/normalizer.rs` |
| 2 | 协议自动检测 | 9Router | `proxy/format_detector.rs` |
| 3 | Tauri 2 + IPC | CC-Switch | 整体架构 |
| 4 | SQLite DAO + 迁移 | CC-Switch | `storage/` |
| 5 | MCP 攻击链 | Pipelock | `security/mcp_chains.rs` |
| 6 | MCP 工具基线 | Pipelock | `security/mcp_baseline.rs` |
| 7 | 失败关闭哲学 | Pipelock | 全局原则 |
| 8 | DLP 先于 DNS | Pipelock | `proxy/pipeline.rs` |

---

*末次更新：2026-05-17 · 责任人：架构组关联：CLAWHEART\_V2\_BLUEPRINT\_CN.md（详细论证） · ui-mocks-v2.md · ui-prototype-v2.html · 6 份竞品分析*
