# 借鉴 pipelock：ClawHeart v2 可行性建议方案

> 作者：调研整合 · 日期：2026-05-19
> 关联里程碑：W5（hudsucker spike）→ W12（基线绿灯）→ W17（云端同步）→ W20（高级能力）
> 关联文档：[`architecture/README.md`](../architecture/README.md)、[`threat-model.md`](../threat-model.md)、[`skill-scanner.md`](../skill-scanner.md)

---

## 0. 摘要（One-Pager）

ClawHeart v2 已在 W1 启动周完成的 8 层防御骨架（归一化 / 危险指令 / 注入 / DLP / MCP chains / 供应链扫描 / Agent 发现 / Kill Switch）在**策略覆盖广度上已对标甚至局部超越 pipelock**，但在以下 4 个维度仍存在结构性短板，这些恰恰是 pipelock 在开源生态中拉开身位的杀手锏：

| 短板 | pipelock 已有 | v2 当前 | 建议借鉴优先级 |
|------|---------------|---------|---------------|
| **可验证审计链** | Ed25519 签名 receipts + hash-chained JSONL flight recorder + 独立 verifier SDK（TS/Rust） | `request_logs` 与 `intercept_events` 表，仅 SQLite，用户可手改 | 🔴 P0 |
| **最低侵入接入入口** | `/fetch?url=...` 子路径 + 标准 HTTPS_PROXY CONNECT 双轨同存 | 仅 hudsucker MITM（W5 才激活，需装 CA） | 🔴 P0 |
| **配置 source of truth** | YAML 单一来源 + fsnotify 热加载 + atomic.Pointer 原子 swap | 策略散落在 17 张 SQLite 表 | 🟠 P1 |
| **可分发的策略包生态** | `pipelock rules install pipelock-community` + 签名 bundle + 社区贡献 | 危险/技能规则内置 Rust 源码 | 🟠 P1 |

此外有 3 个**v2 已实现但实现方式可以更彻底**的点：

| 点 | 当前 v2 实现 | pipelock 启发的强化方向 |
|----|------------|----------------------|
| Scanner pipeline 顺序 | 8 层平铺 | **DNS 解析前置所有敏感扫描**（避免域名查询本身泄露） |
| MCP 双向扫描 | 已有 chain detector + baseline | 补强 **client→server 参数 DLP** 与 **server→client 注入扫描** 的分离 trait 实现 |
| 失败关闭范围 | Kill Switch 与流量层 | 扩展到 **TLS 握手失败 / 上游证书异常 / 解码失败 / panic** 全部默认 block |

本文档的所有建议**不引入 v2 现有路线图之外的新依赖**（hudsucker / rustls / rcgen / tokio / rusqlite / keytar 已具备），只在现有 crate 上做组合扩展。

---

## 1. v2 当前架构快照（基准）

> 证据：`src-tauri/src/proxy/server.rs:22-58`、`security/kill_switch.rs:21-43`、`security/mcp_chains.rs`、`CHANGELOG.md`、`docs/architecture/README.md`

```
┌───────────────────────────────────────────────────────────────────┐
│ Tauri 2 Shell (Rust + React + Tailwind)                            │
│ ┌─ commands/* (40+ IPC)                                            │
│ ├─ proxy/                                                          │
│ │   ├─ formats.rs / format_detector.rs   (5 种 LLM 自动归一化)     │
│ │   ├─ normalizers/{openai,claude,gemini,ollama,responses}.rs      │
│ │   ├─ server.rs  (hudsucker, 19111, W5 spike)                     │
│ │   ├─ handler.rs / streaming.rs / circuit_breaker.rs              │
│ │   └─ usage_extractor.rs / cross_request.rs                       │
│ ├─ security/                                                       │
│ │   ├─ normalizer.rs   (6-pass: zw/nfkc/homoglyph/leet/b64/ws)     │
│ │   ├─ danger.rs       (30 条规则, MITRE 映射)                      │
│ │   ├─ injection.rs    (50 条注入模式)                              │
│ │   ├─ redact.rs       (48 类 DLP + Luhn/Mod97 校验位)              │
│ │   ├─ mcp.rs / mcp_chains.rs (10 链)/ mcp_baseline.rs (漂移)      │
│ │   ├─ skill_scanner.rs (72 条供应链规则 + SkillGuard 评分)         │
│ │   ├─ kill_switch.rs  (4 源 OR: config/api/signal/sentinel)       │
│ │   ├─ advisory.rs     (Ed25519 签名 feed)                          │
│ │   └─ scanner.rs / scanners/* (80 项离线审计)                      │
│ ├─ agents/             (6 平台 Agent 发现 + 漂移)                    │
│ ├─ sync/               (云端同步 worker 骨架, W17+)                  │
│ └─ storage/            (rusqlite + Keychain + 17 表)                │
└───────────────────────────────────────────────────────────────────┘
```

**已对齐 pipelock 的部分**（不必重复借鉴）：
- ✅ 6-pass 归一化（pipelock `scanner/response.go:40-100`）
- ✅ 48 类 DLP（pipelock `internal/redact/providers.go`）
- ✅ Kill Switch + 失败关闭（pipelock kill switch + fail-closed default）
- ✅ MCP 攻击链与基线冻结（pipelock `mcp_baseline` rug-pull detector）
- ✅ 6 协议自动检测（pipelock 只覆盖 OpenAI/Claude/MCP，v2 已超）

---

## 2. P0 建议：可验证审计链（Receipts + Flight Recorder + Verifier SDK）

### 2.1 问题陈述

v2 当前 `request_logs` 与 `intercept_events` 表完全存在本地 SQLite，**用户在 DMG/MSI 安装后可直接用 `sqlite3` 修改记录**。这在以下场景下致命：

1. **合规审计**（SOC2/ISO27001/HIPAA）要求"日志不可篡改"——一条 SQL 就破功
2. **企业版"管理员审计"卖点**——员工可抹掉自己的恶意调用记录
3. **法律取证**——任何修改痕迹都让证据失效

### 2.2 借鉴 pipelock 设计

证据：`pipelock/internal/receipt/action.go:1-75`、`pipelock/internal/receipt/emitter.go`、`pipelock/sdk/audit-packet/`、`pipelock/sdk/verifiers/{ts,rust}/`

```
┌──────────────────────────────────────────────────────────┐
│ 每个拦截决策都产出一条 Action Receipt：                       │
│                                                            │
│  {                                                         │
│    "action_id": "uuid-v7",         ← 时间序                 │
│    "timestamp": "2026-05-19T03:00:00.123Z",                │
│    "verdict": "block",             ← allow/block/warn/...  │
│    "layer": "dlp",                 ← 哪一层命中              │
│    "pattern": "anthropic_key",     ← 规则 ID                │
│    "transport": "forward",         ← forward/mcp/fetch     │
│    "agent": "claude-code",                                 │
│    "target": "https://evil.com/?k=sk-ant-***",  ← 已脱敏    │
│    "request_id": "req-xxx",                                │
│    "previous_hash": "sha256:abc...",  ← hash chain          │
│    "signature": "ed25519:def..."     ← 私钥签名             │
│  }                                                         │
│                                                            │
│  写入 ~/.clawheart/flight/YYYY-MM-DD.jsonl                  │
│  每天一个文件，append-only                                   │
│  签名密钥由首次启动生成，存 Keychain（不可导出）              │
└──────────────────────────────────────────────────────────┘
```

**链式签名**：每条 receipt 的 `previous_hash` 指向上一条 receipt 序列化后的 sha256。任意一条被改 → 后续所有 hash 校验失败。

### 2.3 v2 落地路径

#### Crate 引入（已在 v2 现有依赖谱内）

```toml
# src-tauri/Cargo.toml
[dependencies]
# 已有
sha2 = "0.10"          # v2 已有 security/sha256.rs 自实现，建议替换为标准库
# 新增
ed25519-dalek = { version = "2", features = ["rand_core"] }
serde_jsonlines = "0.7"   # JSONL streaming I/O，零依赖
arc-swap = "1.7"          # 配合后文 §3 配置热加载

[features]
audit_chain = ["dep:ed25519-dalek", "dep:serde_jsonlines"]
```

#### 模块布局

```
src-tauri/src/audit/
├── mod.rs
├── receipt.rs       # ActionReceipt struct + Serialize/Deserialize
├── emitter.rs       # Emitter::new(keychain_signer) / emit(opts) -> Result
├── chain.rs         # FlightRecorder：append + 滚动文件 + tail hash 维护
├── signer.rs        # Keychain 取 Ed25519 私钥；首次启动 generate + 写入
└── exporter.rs      # 导出 .clawheart-evidence.tar.gz（含公钥 + jsonl）
```

#### 关键 trait 与 API

```rust
// src-tauri/src/audit/receipt.rs
#[derive(Serialize, Deserialize, Clone)]
pub struct ActionReceipt {
    pub action_id: String,        // UUID v7（时间有序）
    pub timestamp: String,        // RFC3339 nanos
    pub verdict: Verdict,         // Allow / Block / Warn / Strip / Ask
    pub layer: String,            // "dlp" / "danger" / "injection" / "mcp_chain" / ...
    pub pattern: Option<String>,  // 命中规则 ID
    pub transport: Transport,     // Forward / Fetch / Mcp / WebSocket
    pub agent: Option<String>,
    pub target: String,           // 已脱敏的目标 URL/工具名
    pub request_id: String,
    pub previous_hash: String,    // hex(sha256)
    pub signature: String,        // hex(ed25519)
}

// src-tauri/src/audit/emitter.rs
pub trait ReceiptEmitter: Send + Sync {
    fn emit(&self, opts: EmitOpts) -> Result<ActionReceipt, AuditError>;
    fn tail_hash(&self) -> String;          // 当前链尾 hash（IPC 暴露给 UI）
}
```

#### IPC 命令扩展

```rust
// src-tauri/src/commands/audit.rs（新增）
#[tauri::command]
pub fn export_evidence(range: DateRange, out_path: String) -> AppResult<ExportSummary> { ... }

#[tauri::command]
pub fn get_chain_status() -> AppResult<ChainStatus> {  // 链尾 hash / 公钥指纹 / 总条数
    ...
}

#[tauri::command]
pub fn verify_chain(jsonl_path: String, public_key_hex: String) -> AppResult<VerifyReport> {
    ...
}
```

#### Verifier SDK（独立分发，**对标 pipelock `sdk/verifiers/`**）

新建 monorepo 子项目（不在 src-tauri/ 内）：

```
clawheartv2/
├── sdk/
│   ├── verifier-ts/         # npm @clawheart/verifier
│   │   ├── package.json
│   │   ├── src/index.ts     # 离线验证 jsonl + 公钥
│   │   └── tests/fixtures/  # conformance test vectors
│   └── verifier-rs/         # crates.io clawheart-verifier
│       ├── Cargo.toml
│       └── src/lib.rs       # 同上，CLI: clawheart-verify <jsonl>
```

**关键设计**：verifier 必须**完全脱离 ClawHeart 主程序**——审计员从 GitHub release 下载 `clawheart-verify` 二进制 + 用户提供的 jsonl + 已发布的公钥（公钥指纹也可在 v2 启动横幅 / 设置页常驻显示），输出 `valid | invalid | tampered_at_line_N | bad_signature_at_line_N`。

#### UI 改动（最小）

`src/components/tools/LogsTool.tsx` 加按钮：
- "导出证据包"（弹保存对话框，调 `export_evidence`）
- "查看链尾哈希"（顶部 banner，与 pipelock 的 [verified] 徽章风格一致）
- "公钥指纹"（设置页常驻，便于审计员 OOB 校对）

### 2.4 与 v2 路线图对齐

- **W12 基座绿灯前置**：实现 emitter 与 jsonl writer（不依赖 hudsucker，对所有现有 `commands::intercept` 调用插入 emit 钩子）
- **W17 云端同步配合**：sync worker 可上传 `signed jsonl` 而非原文，云端只存哈希链不存敏感原文，进一步降低合规风险
- **W20 高级能力**：发布 `@clawheart/verifier`（npm）与 `clawheart-verify`（crates.io + GitHub release binary）

---

## 3. P0 建议：双轨接入入口（Fetch 子路径 + 反向代理）

### 3.1 问题陈述

v2 当前**唯一**接入方式是 hudsucker MITM 正向代理（W5 激活）。这意味着：

- 用户必须装 CA 根证书（macOS `security add-trusted-cert`、Windows `certutil`、Linux 各发行版不一致）
- 必须设置系统代理或 `HTTPS_PROXY` 环境变量
- 任何 **TLS pinning 客户端**（部分 Electron App、移动 SDK、企业受控环境）会直接拒绝 ClawHeart CA → 完全逃逸
- 在企业受控终端上"装根证书"本身可能被 MDM 禁止

### 3.2 借鉴 pipelock 设计

证据：`pipelock/cmd/pipelock/main.go`、`internal/cli/runtime/run.go:95-131`、`internal/proxy/proxy.go:2505-2560,2738-2930`、`README.md:46-131`

pipelock 在**同一端口**提供三种入口：

```go
// pipelock/internal/proxy/proxy.go:2505
func (p *Proxy) buildHandler(mux *http.ServeMux) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        if r.Method == http.MethodConnect           { p.handleConnect(...) }      // 标准 HTTPS_PROXY
        if r.URL.IsAbs() && r.URL.Host != ""        { p.handleForwardHTTP(...) }  // 绝对 URI 转发
        mux.ServeHTTP(...)                                                          // /fetch /ws /mcp
    })
}
```

**`/fetch?url=...` 子路径**是关键创新：

- Agent 不发原 HTTPS 请求，而是 `GET http://127.0.0.1:19111/fetch?url=https://api.openai.com/v1/...`
- 无 TLS，无证书，无系统代理设置
- 代理拉取上游 → 扫描响应 → 返回提取后的文本/JSON
- 适合**用户改不了 base_url 但能在 Agent 框架里加 fetch 工具**的场景

### 3.3 v2 落地路径

#### 推荐分端口而非同端口

v2 的 hudsucker 设计以 MITM 为核心，混入子路径会让 hyper 处理器分支复杂化。建议分两个监听器：

```
:19111  hudsucker MITM forward proxy（W5 spike，已规划）
        - CONNECT 隧道
        - 绝对 URI 转发
        - 需装 ClawHeart CA

:19112  ClawHeart Fetch API（建议本提案新增，纯 hyper/axum，零 TLS）
        - GET  /fetch?url=...&format=auto
        - POST /chat/completions  ← OpenAI 兼容子路径
        - POST /messages          ← Anthropic 兼容子路径
        - POST /mcp               ← MCP JSON-RPC over HTTP
        - 客户端→代理是 HTTP，代理→上游是 HTTPS
```

#### Cargo 依赖

axum 已在 Tauri 2 间接依赖中（hudsucker 用 hyper），无需新增重量。

```toml
[dependencies]
axum = { version = "0.7", features = ["json", "macros"] }
tower-http = { version = "0.5", features = ["cors", "limit"] }
```

#### 模块布局

```
src-tauri/src/proxy/
├── server.rs            # 现有 hudsucker（:19111）
├── fetch_server.rs      # 新增 axum :19112
├── routes/
│   ├── fetch.rs         # GET /fetch?url=...
│   ├── openai_compat.rs # POST /v1/chat/completions  → 转发归一化 → 上游
│   ├── claude_compat.rs # POST /v1/messages
│   ├── mcp_http.rs      # POST /mcp（JSON-RPC over HTTP）
│   └── health.rs        # GET /healthz / /metrics
└── pipeline.rs          # 共享：两入口都走同一 SecurityPipeline
```

**关键点**：fetch_server 与 hudsucker 共享同一 `SecurityPipeline`，scanner 实现 0 重复。

#### 兼容子路径设计（保留 v1 的"自适应 LLM 响应格式拦截"）

v1 最大的差异化体验——**block 时不返回 HTTP 403，而是伪造完整 LLM 响应**——必须在 v2 完整保留。建议在 `routes/openai_compat.rs` 中：

```rust
// 当 SecurityPipeline 返回 Verdict::Block 时
match request_format {
    LlmFormat::OpenAI if body.stream => respond_openai_sse_block(msg).await,
    LlmFormat::OpenAI => respond_openai_json_block(msg).await,
    LlmFormat::Claude => respond_anthropic_sse_block(msg).await,
    LlmFormat::Gemini => respond_gemini_sse_block(msg).await,
    LlmFormat::Ollama => respond_ollama_chunk_block(msg).await,
    _ => respond_generic_block(msg).await,
}
```

这是 v2 相对 pipelock 的**核心 UX 护城河**——pipelock 一律返回 HTTP 403，破坏 Claude Code / Cursor / Cline 的 chat UI；v2 让拦截信息出现在 chat bubble 里。

#### 三档接入模式（用户视角）

复用 v2 现有 UI 框架，在 `SettingsTool` 加一个三段滑块：

| 档位 | 监听 | 用户操作 | 适用 |
|------|------|---------|------|
| **零信任档**（默认） | `:19112` | 改 `OPENAI_API_BASE=http://127.0.0.1:19112/v1` | 试用、个人 |
| **审计档** | `:19112 + :19111` | 一键装 CA + 设系统代理（UI 引导） | 开发者 |
| **强制档** | + sandbox 子命令 | `clawheart sandbox -- python agent.py` | 企业、合规 |

### 3.4 与 v2 路线图对齐

- **W5 hudsucker spike 同期**：fetch_server 用 axum 独立实现，可与 hudsucker 并行开发，互不阻塞
- **W12 基座绿灯**：fetch_server 与 hudsucker 共用 `SecurityPipeline`，UI 三档滑块上线
- 文档化盲区：明确告知用户 TLS pinning 客户端在审计档下仍可能逃逸；这是诚实边界

---

## 4. P1 建议：YAML Source of Truth + 配置热加载

### 4.1 问题陈述

v2 当前所有策略（30 条危险指令、50 条注入模式、48 类 DLP、72 条供应链规则、4 源 Kill Switch 配置）的**单一可信源**散落在：

- Rust 源码 const（`security/danger.rs`、`injection.rs`、`redact.rs`）
- SQLite 17 张表（`storage/migrations.rs` + seed）
- Tauri config（`tauri.conf.json`）

三处不同步，导致：

1. 用户无法直接编辑策略（必须改 SQLite 或重新编译）
2. 企业策略无法 Git 化版本控制
3. 策略更新需要应用重启或 sync 表更新

### 4.2 借鉴 pipelock 设计

证据：`pipelock/configs/balanced.yaml`、`pipelock/configs/claude-code.yaml`、`pipelock/CLAUDE.md:115-117`、`pipelock/internal/config/reload.go`

pipelock 把所有策略写入单一 YAML，启动时加载到内存，**fsnotify 监听变更 + SIGHUP**：

```go
// pipelock 用 atomic.Pointer[T] 原子替换
cfgPtr := atomic.Pointer[Config]{}
scannerPtr := atomic.Pointer[Scanner]{}
// 文件变更回调
watcher.Events <- event:
    newCfg, _ := loadYAML(path)
    newScanner := buildScanner(newCfg)
    cfgPtr.Store(&newCfg)        // 原子替换
    scannerPtr.Store(&newScanner)
    // ~100ms debounce
```

任何正在执行的请求继续用旧 cfg/scanner，新请求自动用新版。**零停机、零进程重启**。

### 4.3 v2 落地路径

#### Crate 引入

```toml
[dependencies]
serde_yaml = "0.9"
notify = "6"             # Rust fsnotify
arc-swap = "1.7"         # 对应 Go 的 atomic.Pointer
```

#### 文件布局

```
~/.clawheart/
├── policy.yaml           # 用户可编辑（编辑器或 UI 写入）
├── policy.lock           # 启动时校验 hmac，防外部进程篡改
├── ca.pem / ca.key       # CA（Keychain 备份）
└── flight/
    └── 2026-05-19.jsonl  # §2 飞行记录
```

#### policy.yaml 示例（v2 风格）

```yaml
version: 2
mode: balanced  # strict / balanced / audit-only

listeners:
  forward_proxy:
    enabled: false
    listen: "127.0.0.1:19111"
    ca_auto_install: true
  fetch_server:
    enabled: true
    listen: "127.0.0.1:19112"

policies:
  allowlist_domains:
    - "api.openai.com"
    - "api.anthropic.com"
    - "generativelanguage.googleapis.com"
  blocklist_domains: []

  # 引用而非内联：减小文件，便于 community 包覆盖
  danger_rules_from: "builtin:v2"      # 内置 30 条，或 path: "./rules/danger.yaml"
  injection_rules_from: "builtin:v2"
  dlp_patterns_from: "builtin:v2"
  skill_rules_from: "builtin:v2"

  mcp:
    chain_detection: true
    baseline_freeze: true
    drift_action: warn   # warn / block

  budget:
    daily_usd: 10
    monthly_usd: 100

audit:
  receipts_enabled: true
  flight_recorder_dir: "~/.clawheart/flight"

kill_switch:
  sentinel_path: "~/.clawheart/STOP"
  config_kill: false
```

#### 模块布局

```
src-tauri/src/config/
├── mod.rs
├── schema.rs           # serde struct 反序列化
├── loader.rs           # load_yaml(path) -> Result<Config>
├── watcher.rs          # notify::RecommendedWatcher + debounce
├── atomic_store.rs     # ArcSwap<Config> + ArcSwap<SecurityPipeline>
└── lock.rs             # HMAC-SHA256 校验 policy.lock
```

#### 与现有 SQLite 表的分工

YAML 是 **source of truth**，SQLite 是 **查询索引 + 运行时状态**：

| 数据 | YAML（不可变规则） | SQLite（运行时） |
|------|---------------------|------------------|
| 危险指令规则 | ✅ | 缓存（查询加速） |
| 用户禁用技能列表 | ✅ | 缓存 |
| 已发现的 Agent | ❌ | ✅ |
| 拦截事件 / 请求日志 | ❌ | ✅ |
| Token 用量统计 | ❌ | ✅ |
| 公告与漂移检测状态 | ❌ | ✅ |

**启动流程**：
1. 读 YAML → 构建 SecurityPipeline → ArcSwap 存
2. 把 YAML 内的规则**单向同步**到 SQLite cache 表（用于 UI 列表查询）
3. fsnotify 触发：重新解析 → 重建 pipeline → ArcSwap 原子替换 → 重新同步 cache

**UI 编辑路径**：
- "添加自定义危险指令" UI 操作 → 写入 `~/.clawheart/policy.yaml` 的 `extra_danger_rules` 段 → fsnotify 自动触发热加载
- 也支持只写到 SQLite 的 `user_overrides` 表（不污染 YAML），运行时把两边 merge

### 4.4 与 v2 路线图对齐

- **W12 基座绿灯前置**：完成 loader + watcher + atomic_store
- **W14 i18n 完成同期**：UI 编辑表单写入 YAML
- **W17 云端同步**：sync worker 从云端拉的策略更新落到 `~/.clawheart/policy.yaml.upstream`，由 watcher 合并到主 YAML

---

## 5. P1 建议：可分发的策略包生态（Rule Bundle）

### 5.1 问题陈述

v2 当前 30 条危险指令 / 50 条注入模式 / 72 条供应链规则全部硬编码在 Rust 源文件。后续问题：

1. 安全研究员发现新攻击 → 需要等 ClawHeart 主版本更新才能拿到
2. 企业有内部敏感规则（如公司内域名、内部 API key 格式）→ 无标准接入方式
3. 社区贡献规则无渠道

### 5.2 借鉴 pipelock 设计

证据：`pipelock/README.md:399-407` —— `pipelock rules install pipelock-community`

pipelock 把 rule bundle 设计为 **签名 tar.gz**：

```
pipelock-community-2026-05-19.tar.gz
├── manifest.json       # 版本、签名公钥指纹、规则数量
├── danger.yaml         # 危险指令补充
├── injection.yaml      # 注入模式补充
├── dlp.yaml            # DLP 模式补充
└── signature.bin       # Ed25519 签名（覆盖前 4 个文件的 sha256）
```

`pipelock rules install <name>` → 下载 + 验签 + 解压到 `~/.pipelock/rules/<name>/` → 在 `policy.yaml` 内 `rules.bundles: ["pipelock-community"]` 引用。

### 5.3 v2 落地路径

#### 命名与命令

利用 v2 已有 Tauri CLI 入口：

```bash
clawheart rules install clawheart-community
clawheart rules install enterprise@~/Downloads/acme-internal.tar.gz
clawheart rules list
clawheart rules remove enterprise
clawheart rules verify clawheart-community
```

或 UI 操作：`SkillsTool` 加"添加规则包"按钮，文件选择器 + URL 输入。

#### 模块布局

```
src-tauri/src/rules_bundle/
├── mod.rs
├── manifest.rs       # ManifestV1 struct
├── installer.rs      # 下载 / 校验 / 解压 / 注册
├── verifier.rs       # Ed25519 验签（复用 §2 audit 模块）
└── registry.rs       # 已安装包元数据
```

#### 公开规则源

第一阶段：clawheart-community 作为 ClawHeart 官方维护的 GitHub 仓库：

```
github.com/clawheart/clawheart-rules-community/
├── danger/      # YAML 文件，每周更新
├── injection/
├── dlp/
├── skills/
└── releases/    # 签名 tar.gz
```

发布流程：CI 用项目签名密钥（GitHub Secrets）打包 + 签名 + 自动 release。

### 5.4 与 v2 路线图对齐

- **W17 云端同步同期**：sync worker 已有同步 advisories 的能力，扩展到 rule bundle
- **W20 高级能力**：开放第三方贡献接受 PR + 维护规则仓库

---

## 6. P1 建议：强化 Scanner Pipeline 顺序

### 6.1 问题陈述

v2 当前 8 层防御**平铺**调用，但 pipelock 的 14 层 pipeline 有一个**关键设计**：所有"读取/扫描敏感模式"的检查**必须放在 DNS 解析之前**，否则域名查询本身就是一次外泄（DNS 服务器记录到 `evil-exfil-abc123.attacker.com`）。

### 6.2 借鉴 pipelock 设计

证据：`pipelock/internal/scanner/scanner.go:800-920`

```
扫描顺序（必须按此序）：
1. scheme           ← O(1)
2. CRLF / 路径穿越  ← O(1)
3. allowlist        ← 字符串匹配，O(1)
4. blocklist        ← 字符串匹配，O(1)
5. core SSRF literal（如 169.254.169.254 元数据服务）← 字符串
6. core DLP（不可关闭的安全底线 48 条）              ← 正则
7. DLP + entropy（用户规则）                          ← 正则 + 熵
8. subdomain entropy（DNS 隧道检测）                  ← 字符串熵
─────────────────────────────────────────────────────
9. SSRF（DNS 解析，第一次真正触发网络 I/O）          ← DNS lookup
─────────────────────────────────────────────────────
10. rate limit / data budget / URL length            ← 状态查询
```

**第 9 步之前不能有任何 I/O**——任何 DNS 查询都会通过 stub resolver 上报 ISP / 系统 DNS 服务。

### 6.3 v2 落地路径

修改 `src-tauri/src/proxy/pipeline.rs`：

```rust
pub async fn evaluate(&self, req: &NormalizedRequest) -> Verdict {
    // PHASE 1: 不触发任何 I/O 的检查
    self.scheme_check(req)?;
    self.crlf_path_check(req)?;
    self.allowlist_check(req)?;
    self.blocklist_check(req)?;
    self.core_dlp_check(req)?;          // 不可关闭的底线
    self.danger_command_check(req)?;
    self.injection_check(req)?;
    self.user_dlp_check(req)?;
    self.entropy_check(req)?;
    self.subdomain_entropy_check(req)?;

    // PHASE 2: 仅在 phase 1 全过后才允许网络 I/O
    self.ssrf_check_with_dns(req).await?;  // DNS 在此发生
    self.rate_limit_check(req)?;
    self.budget_check(req)?;

    Verdict::Allow
}
```

在 trait 定义层面强制：

```rust
pub trait SecurityCheck: Send + Sync {
    fn category(&self) -> CheckCategory;  // Phase1NoIO / Phase2WithIO
    fn evaluate(&self, ctx: &CheckCtx) -> CheckOutcome;
}

// PipelineBuilder 自动按 category 排序，禁止 Phase2 检查放在 Phase1 之前
```

### 6.4 与 v2 路线图对齐

- **W8 安全引擎核心收尾前**：完成 pipeline 重排
- 不引入新依赖，纯重构 + 单元测试

---

## 7. P1 建议：MCP 双向扫描的 trait 分离

### 7.1 问题陈述

v2 的 `security/mcp.rs` + `mcp_chains.rs` + `mcp_baseline.rs` 已具备良好基础，但当前 MCP 扫描入口集中在一个 `MCPInterceptor`，**Client→Server 与 Server→Client 两个方向的策略集合未明确分离**。

实际上：

- **Client→Server（参数 → 工具）**：主要风险是**外泄密钥**（agent 把 `.env` 文件内容传给 web tool）→ 用 DLP 规则
- **Server→Client（工具结果 → agent）**：主要风险是**prompt injection**（tool 返回的 markdown 里藏指令）→ 用 6-pass 归一化 + injection 规则

### 7.2 借鉴 pipelock 设计

证据：`pipelock/internal/mcp/proxy.go:1-100`

```go
type Proxy struct { ... }

func (p *Proxy) ForwardScannedInput(msg []byte) error {
    if req.Method == "tools/call" {
        // 仅 DLP 扫描参数
        dlpResult := p.scanner.Scan(ctx, string(req.Params))
        if !dlpResult.Allowed { return err }
    }
    return p.upstream.Write(msg)
}

func (p *Proxy) ScanResponseOneway(ctx context.Context, msg []byte) ([]byte, error) {
    for _, c := range resp.Result.Content {
        // 仅 injection 扫描结果
        injResult := p.scanner.ScanResponse(ctx, c.Text)
        if !injResult.Clean { return nil, err }
    }
    return msg, nil
}
```

两个方法独立、规则集独立、错误处理独立。

### 7.3 v2 落地路径

```rust
// src-tauri/src/security/mcp/
├── mod.rs
├── direction.rs
│
trait McpDirectionScanner {
    fn direction(&self) -> McpDirection;  // ClientToServer / ServerToClient
    fn scan(&self, msg: &McpMessage) -> Verdict;
}

pub struct ClientToServerScanner {
    dlp: DlpScanner,
    chain_detector: Arc<ChainDetector>,
    baseline: Arc<ToolBaseline>,
}

pub struct ServerToClientScanner {
    injection: InjectionScanner,
    normalizer: Normalizer6Pass,
    baseline: Arc<ToolBaseline>,
}
```

链检测放在 `ClientToServerScanner` 中（基于工具调用序列）。
基线漂移在双向都检查（工具描述变更可能被服务端 push）。

### 7.4 与 v2 路线图对齐

- **W8 安全引擎核心收尾**：拆分 mcp 模块的 trait
- **W9 Armorer Guard 决策**：如果决定集成 Armorer Guard，作为 Client→Server 的可选异步扫描层

---

## 8. P2 建议：失败关闭范围扩展

### 8.1 问题陈述

v2 的失败关闭目前主要在 Kill Switch 与流量层（`security/kill_switch.rs:36-42` 中的 `Err(_) => true`），但**其他错误路径未统一**：

- TLS 握手失败 → 当前可能直接 502
- 上游证书异常 → 当前可能 502
- 协议解码失败（非标准 JSON） → 当前可能 502
- panic 捕获后默认行为不确定

### 8.2 借鉴 pipelock 设计

pipelock 在 `internal/proxy/proxy.go` 的所有错误分支统一调用 `writeBlockedJSON` 或 `writeBlockedError`，**绝不返回 502**——这避免了"代理错误被 agent 当成网络抖动 → retry → 部分流量绕过"的攻击面。

### 8.3 v2 落地路径

定义统一的 error → response 映射：

```rust
// src-tauri/src/proxy/error_handler.rs
pub fn handle_proxy_error<F: LlmFormat>(err: ProxyError, format: F) -> Response {
    // 所有错误路径都构造一个 "blocked" 风格的响应
    // 而非 hyper 默认的 502
    match err {
        ProxyError::TlsHandshake(_) |
        ProxyError::UpstreamCertInvalid(_) |
        ProxyError::DecodingFailed(_) |
        ProxyError::InternalPanic(_) => {
            respond_blocked(format, "ClawHeart 安全引擎拒绝处理此请求")
        }
        ProxyError::NetworkTimeout(_) => {
            respond_blocked(format, "上游网络超时；为安全起见已阻断")
        }
    }
}
```

并在 `tower-http` 层加 panic catcher → 同样走 blocked 响应。

### 8.4 与 v2 路线图对齐

- **W12 基座绿灯前**：完成统一 error_handler
- 加 fuzz test：构造畸形请求触发各种错误路径，断言**永远不返回 502**

---

## 9. P2 建议：Sandbox 强制模式（macOS / Linux / Windows）

### 9.1 问题陈述

v2 当前最强档（"强制档"）需要用户主动配置系统代理 + 装 CA，仍存在用户疏漏。pipelock 提供的 `pipelock sandbox -- python agent.py` 是更彻底的解决方案：**通过 OS 沙箱机制强制进程的所有网络流量经过 ClawHeart**，连环境变量都不用设。

### 9.2 借鉴 pipelock 设计

证据：`pipelock/README.md:46-131` —— Landlock + seccomp + netns

```bash
pipelock sandbox --config pipelock.yaml -- python agent.py
```

实现方式：
- Linux：Landlock（文件 R/W 限制） + seccomp-bpf（系统调用过滤） + netns（强制流量走 ClawHeart）
- macOS：`sandbox-exec` + `seatbelt` profile（限制 socket → 仅 127.0.0.1）
- Windows：AppContainer + WFP（Windows Filtering Platform）规则

### 9.3 v2 落地路径

实现独立 CLI 子命令（Tauri 2 已经支持 binary 暴露）：

```
src-tauri/src/cli/
├── mod.rs
├── sandbox/
│   ├── mod.rs
│   ├── linux.rs       # landlock + seccomp，依赖 landlock crate
│   ├── macos.rs       # sandbox-exec + seatbelt profile 模板
│   └── windows.rs     # WFP API 绑定 + AppContainer
└── run.rs             # 子进程启动 + 信号转发
```

**关键设计**：sandbox 实现只在**强制档启用**才编译进二进制（`#[cfg(feature = "sandbox")]`），默认 feature 关闭以保持 v2 包体积 <15MB。

### 9.4 与 v2 路线图对齐

- **W20 高级能力**：完成 Linux 与 macOS 实现
- **GA 之后 v2.1**：Windows WFP
- 这是企业版差异化定位的关键功能（"管理员一键强制员工流量审计"）

---

## 10. 总体落地排序

按 v2 现有里程碑插入：

```
W5  hudsucker spike  ─┬─ §3 fetch_server 平行开发（axum :19112）
                      └─ §8 失败关闭统一 error_handler（重构）
                      
W8  安全引擎核心收尾 ──┬─ §6 pipeline 顺序重排（trait phase 强制）
                      └─ §7 MCP 双向 scanner trait 分离

W12 基座绿灯       ──┬─ §2 receipts + flight recorder 集成
                      └─ §4 YAML source of truth + 热加载

W14 i18n 完成      ── §4 UI 编辑表单写入 YAML

W17 云端同步       ──┬─ §2 verifier SDK 发布（npm + crates.io）
                      └─ §5 rule bundle installer

W20 高级能力       ──┬─ §5 clawheart-community 仓库与 CI 自动发布
                      └─ §9 sandbox 子命令（Linux + macOS）

GA (2026-11-01)    ── 全部 P0 + P1 完成；P2 sandbox Windows 留 v2.1
```

---

## 11. 风险与缓解

| 风险 | 缓解 |
|------|------|
| Ed25519 签名密钥在 Keychain 中被恶意 App 读取 | 用 macOS 的 `kSecAccessControlBiometryAny` 强制 Touch ID 解锁；Linux 用 libsecret + GNOME Keyring 锁屏自动锁定 |
| 飞行记录 jsonl 文件被 root 进程改写 | 文件 chmod 600 + 周期性把 tail hash 上报云端做 OOB 比对（W17 sync 配合） |
| YAML 热加载期间正在执行的请求看到的策略版本不一致 | ArcSwap 保证原子性；请求开始即 Load 一次，整个生命周期用同一版本 |
| rule bundle 公钥被替换（供应链攻击） | 公钥指纹硬编码在 Rust 源码中，配合 CI gitleaks；用户安装时显式确认公钥指纹 |
| sandbox 强制模式在 macOS 系统更新后失效 | CI 矩阵覆盖 macOS 14/15/16 三版本；启动时自检 sandbox-exec 可用性，失效则降级到审计档并提示用户 |
| axum :19112 与 hudsucker :19111 同时监听端口冲突 | settings 校验端口空闲；冲突时自动 +1 探测；UI 提示用户 |

---

## 12. 与 v1 / pipelock / 行业的最终定位

```
                  覆盖广度
                     ▲
                     │
     pipelock ●      │      ● ClawHeart v2 (目标)
                     │
                     │
                     │
     ClawHeart v1 ●  │
                     │
   LiteLLM Proxy   ● │  ● Helicone / Langfuse
                     │
                     └──────────────────────────► 桌面用户易用性
                          低                  高
```

- **pipelock**：覆盖最广，但面向 DevOps，CLI/YAML 操作门槛高
- **LiteLLM Proxy / Helicone**：易用但覆盖窄（无 MCP、无供应链扫描）
- **ClawHeart v1**：用户体验好（自适应响应格式）但拦截覆盖薄
- **ClawHeart v2 目标态**：**把 pipelock 的拦截广度翻译成 Tauri 桌面的 GUI 体验**——这是借鉴本文档的全部建议后能占住的生态位

---

## 13. 不建议借鉴的部分

为避免引入复杂度但收益有限的项目，以下 pipelock 设计**不建议借鉴**：

| pipelock 设计 | 不借鉴理由 |
|--------------|----------|
| Go `internal/cli/runtime/run.go` 的多模式同端口分发 | hudsucker（Rust）天然适合分端口；同端口混入子路径让 hyper 分支复杂 |
| GitHub Actions `pipelock-agent-egress-action` 集成 | v2 是桌面产品，CI/CD 场景非主战场（v2.x 可作为附加产品发布） |
| ELv2 企业版双协议 | v2 已规划企业版功能，但开源协议建议 MIT/Apache 2.0 单一，避免双协议带来的合规复杂度 |
| 8-bucket verdict tally（allow/block/warn/ask/strip/forward/redirect/other） | v2 当前 5 类（Allow/Block/Warn/Strip/Ask）已够用；额外的 forward/redirect/other 增加 UI 复杂度 |
| OPA/Rego/CEL 策略引擎 | YAML + 内置规则集对 v2 用户画像足够，引入 OPA 让规则学习曲线陡峭 |

---

## 14. 附：v2 已超越 pipelock 的部分（保持）

```
✅ 5 种 LLM 协议自动归一化（pipelock 仅显式列 OpenAI/Claude/MCP）
✅ 72 条技能供应链规则 + SkillGuard 评分（pipelock 无此层）
✅ 6 平台 Agent 自动发现 + 漂移检测（pipelock 不做发现）
✅ 5 主题 + ⌘K 命令面板 + 应用内托盘弹窗（pipelock 无 UI）
✅ Tauri 系统集成（Keychain / 托盘 / 通知）（pipelock 是 CLI 优先）
✅ 拦截响应自适应 LLM 格式（pipelock 一律 HTTP 403）
✅ <15MB 包 / <40MB 内存 / <1s 启动（pipelock Go 二进制 ~50MB）
```

**结论**：v2 的护城河不是"比 pipelock 更全"，而是"把 pipelock 的全翻译成桌面用户能直接用的 GUI"。本文档全部建议都服务于这一定位。

---

## 附录 A：建议引入的新 Crate 汇总

```toml
# src-tauri/Cargo.toml [dependencies]
ed25519-dalek = { version = "2", features = ["rand_core"] }   # §2 签名 receipts
serde_jsonlines = "0.7"                                        # §2 jsonl I/O
serde_yaml = "0.9"                                             # §4 YAML 解析
notify = "6"                                                   # §4 fsnotify
arc-swap = "1.7"                                               # §4 原子 swap
axum = { version = "0.7", features = ["json", "macros"] }      # §3 fetch_server
tower-http = { version = "0.5", features = ["cors", "limit", "catch-panic"] }  # §3 + §8
landlock = { version = "0.4", optional = true }                # §9 Linux sandbox

[features]
default = ["storage"]
audit_chain = ["dep:ed25519-dalek", "dep:serde_jsonlines"]
config_yaml = ["dep:serde_yaml", "dep:notify", "dep:arc-swap"]
fetch_server = ["dep:axum", "dep:tower-http"]
sandbox = ["dep:landlock"]
```

总包体积增量预估：**+3.2MB**（主要来自 axum + ed25519-dalek + landlock），仍可控制在 <15MB 目标内。

## 附录 B：建议新增 IPC 命令清单

```rust
// 审计链相关
commands::audit::export_evidence(range, out_path) -> ExportSummary
commands::audit::get_chain_status() -> ChainStatus
commands::audit::verify_chain(jsonl, pubkey) -> VerifyReport

// 配置相关
commands::config::reload() -> Result<()>             // 手动触发
commands::config::get_config_path() -> PathBuf
commands::config::write_user_override(rule) -> ()

// Fetch server 相关
commands::proxy::get_fetch_endpoint() -> String      // 给 UI 显示当前 URL
commands::proxy::set_listener_mode(mode) -> ()       // 零信任/审计/强制

// Rule bundle 相关
commands::rules::install(source) -> InstallResult
commands::rules::list() -> Vec<BundleMeta>
commands::rules::remove(name) -> Result<()>
commands::rules::verify(name) -> VerifyResult

// Sandbox 相关
commands::sandbox::available_modes() -> Vec<SandboxMode>
commands::sandbox::launch(cmd, args) -> ChildHandle
```

## 附录 C：v2 文档目录建议结构（演进后）

```
docs/
├── api.md                          # 现有，扩展附录 B 新命令
├── architecture/
│   └── README.md                   # 现有，新增 §receipts / §yaml-config 章节
├── threat-model.md                 # 现有，新增"审计链不可篡改"威胁应对
├── skill-scanner.md                # 现有
├── migration/
│   └── v1-to-v2.md                # 现有，新增 §v1-keychain-to-v2-flight-recorder
├── ui/
│   └── README.md                   # 现有
├── ui-states.md                    # 现有
├── proposals/                      # 新增（本文档所在）
│   └── borrow-from-pipelock.md
└── references/                     # 新增（pipelock 等外部方案速查）
    └── pipelock-architecture.md
```
