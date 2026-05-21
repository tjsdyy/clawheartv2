# Agent 配置一键覆盖（第三方 LLM 中转 API 集中管理）

> 关联：[`access-mode-ui-design.md`](./access-mode-ui-design.md) §3 反向代理接入 / [`borrow-from-pipelock.md`](./borrow-from-pipelock.md) §2.4 虚拟 Key
> 状态：设计阶段（pending review）· 拟纳入 W7 Provider Profile 模块
> 目标：解决用户在多个 AI 工具间重复配置 "第三方中转 API base_url + api_key" 的繁琐流程

---

## 0. 问题陈述（Why）

### 0.1 当前用户的真实痛点

主流 AI 开发者通常**同时使用多个工具**（Cursor / Claude Code / Codex CLI / Cline / Continue / Aider …），且**普遍使用第三方中转/聚合 API**（OpenRouter / DeepBricks / OneAPI / NewAPI / Azure OpenAI / 自托管 LiteLLM 等），原因：

1. 节省成本（聚合定价 < 直连 OpenAI/Anthropic）
2. 突破地域限制
3. 统一计费 / 团队配额
4. 内部合规网关

每次更换中转站或更新 API key，用户需要**逐个工具修改配置**：

| 工具 | 配置位置 | 修改方式 |
|------|---------|---------|
| Cursor | `~/Library/Application Support/Cursor/User/settings.json` 或 GUI | 改 `cursor.openaiBaseUrl` / `cursor.openaiApiKey` |
| Claude Code | `~/.claude/settings.json` / env `ANTHROPIC_BASE_URL` | 改 base_url / 改 API_KEY |
| Codex CLI | shell rc 文件 / `~/.config/openai/config.toml` | 改环境变量 |
| Cline (VS Code) | VS Code settings UI / `.vscode/settings.json` | GUI 设置 |
| Continue.dev | `~/.continue/config.json` | JSON 手编辑 |
| Aider | 命令行参数 / 环境变量 / `~/.aider.conf.yml` | 三种之一 |
| 其他自定义脚本 | OPENAI_API_BASE / ANTHROPIC_BASE_URL env | shell 配置 |

**实际影响**：
- 一次切换中转站 ≈ 5-10 处文件 / 环境变量改动
- 中转站 API Key 散落在 ≥ 10 个位置，泄露风险高
- 没有回滚机制，改错就要重新去查每个工具的文档
- 多用户机器（家庭/工作）凭据相互污染

### 0.2 现有 ClawHeart v2 的不足

v2「基础监控」模式要求用户把工具 base_url 指向 `127.0.0.1:19112`，**这相当于又多了一处配置**。如果不提供自动化工具，反而加剧了上述问题。

---

## 1. 解决方案核心（What）

> **一个集中的 Provider Profile 仓库 + 一键覆盖到已发现 Agent + 完整可回滚的快照机制 + 虚拟 Key 隔离真实凭据。**

```
旧方式（10+ 处分散）                  →     新方式（1 处集中 + 一键 N 处下发）
─────────────────────────────────         ──────────────────────────────────
Cursor settings.json                       ClawHeart
  baseUrl: openrouter.ai/api/v1            ┌─────────────────────────────┐
  apiKey:  sk-or-xxxxxxxxxxxx              │ Provider Profile 仓库         │
                                           │  ┌────────────────────────┐ │
Claude Code .claude/config.json            │  │ Profile A: OpenRouter   │ │
  ANTHROPIC_BASE_URL: openrouter.ai        │  │  baseUrl, apiKey, ...   │ │
                                           │  └────────────────────────┘ │
Codex env                                  │  ┌────────────────────────┐ │
  OPENAI_API_BASE: openrouter.ai...        │  │ Profile B: Azure        │ │
                                           │  └────────────────────────┘ │
Cline .vscode/settings.json                └──────────────┬──────────────┘
  customBaseUrl: openrouter.ai                            │
                                                          ▼ 一键覆盖
Continue config.json                       ┌──────────────────────────────┐
  ...                                       │ 已发现 Agent ──→ ClawHeart    │
                                            │   Cursor      base=127:19112 │
   每个文件都裸存真实 key                     │   Claude Code base=127:19112 │
                                            │   Codex       base=127:19112 │
                                            │   ...         key=sk-claw-xxx│
                                            └──────────────────────────────┘
                                            真实 OpenRouter key 锁在 Keychain
                                            Agent 只能看到 sk-claw-xxx 虚拟 key
```

---

## 2. 数据模型

### 2.1 Provider Profile

```rust
// src-tauri/src/storage/models.rs (新增)
pub struct ProviderProfile {
    pub id: String,                  // uuid v7
    pub name: String,                 // 用户给的名字："OpenRouter 主账号"
    pub provider_kind: String,        // openrouter / azure / deepbricks / newapi / openai / anthropic / litellm / custom
    pub protocol: String,             // openai / anthropic / gemini / ollama / openai_responses
    pub base_url: String,             // https://openrouter.ai/api/v1
    pub credential_ref: String,       // Keychain 里的 key 引用（不存明文）
    pub default_model: Option<String>,// 可选默认模型
    pub headers: HashMap<String, String>,  // 附加头（如 X-Provider-Token）
    pub virtual_key: String,          // sk-claw-{uuid}  对外发的虚拟 key
    pub is_default: bool,
    pub enabled: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
```

### 2.2 Agent Config Snapshot（回滚机制）

```rust
pub struct AgentConfigSnapshot {
    pub id: String,
    pub agent_id: String,             // 关联 agents 表
    pub agent_platform: String,        // claude / cursor / codex / cline / ...
    pub config_path: String,           // 实际写入的文件路径
    pub config_kind: String,           // file_patch / env_var / vsx_setting
    pub before_value: String,          // 原始值（JSON 序列化）
    pub after_value: String,           // 覆盖后的值
    pub applied_at: DateTime,
    pub batch_id: String,              // 一次"一键覆盖"操作的 batch id（同时改的多条共享同一 batch）
    pub rolled_back_at: Option<DateTime>,
}
```

### 2.3 Active Routing（运行时路由表）

```rust
pub struct ActiveRouting {
    pub agent_id: String,              // 哪个 Agent
    pub profile_id: String,            // 解析到哪个 Profile
    pub virtual_key: String,           // 该 Agent 看到的 sk-claw-xxx
}
```

代理收到请求时：根据 `virtual_key`（在 `Authorization` 头中）反查 Profile → 用真实 `credential_ref` 解密 Keychain → 转发到 `base_url`。

---

## 3. Agent Config Probe（探测器）

为每个支持的 Agent 平台实现一个 trait：

```rust
// src-tauri/src/agents/config_probe.rs
pub trait ConfigProbe: Send + Sync {
    /// 平台标识
    fn platform(&self) -> &'static str;

    /// 探测当前 Agent 的配置
    /// 返回：当前 base_url / key 来源 / 是否可写
    fn inspect(&self, agent: &DiscoveredAgent) -> ProbeResult;

    /// 计算如何把配置改为指向 ClawHeart
    fn plan_overwrite(
        &self,
        agent: &DiscoveredAgent,
        target: &OverwriteTarget,
    ) -> Vec<ConfigPatch>;

    /// 实际写入（含备份）
    fn apply(&self, patch: &ConfigPatch) -> Result<AgentConfigSnapshot>;

    /// 从快照恢复
    fn rollback(&self, snapshot: &AgentConfigSnapshot) -> Result<()>;
}

pub struct ProbeResult {
    pub agent_id: String,
    pub current_base_url: Option<String>,
    pub current_key_present: bool,
    pub config_source: ConfigSource,        // file_path / env_var / unknown
    pub writable: bool,
    pub warnings: Vec<String>,              // "需要权限"、"配置受 MDM 管控"
}

pub enum ConfigSource {
    JsonFile { path: String, json_path: String },     // e.g. ~/.continue/config.json → $.models[0].apiBase
    TomlFile { path: String, key: String },
    EnvVar   { name: String, scope: EnvScope },        // shell rc / launchd plist / .pam_environment
    VsCodeWorkspace { path: String, setting: String },
}

pub struct ConfigPatch {
    pub agent_id: String,
    pub platform: String,
    pub source: ConfigSource,
    pub before: String,
    pub after: String,
    pub risk_level: PatchRisk,         // Safe / Caution / Risky
}

pub enum PatchRisk {
    Safe,         // 改 JSON / TOML 文件，备份完整可回滚
    Caution,      // 改 shell rc / 环境变量（用户重启 shell 才生效）
    Risky,        // 改受 MDM/sudo 保护的配置
}
```

### 3.1 各平台 Probe 实现要点

| Platform | 配置源 | 备份策略 |
|----------|--------|---------|
| Cursor | `~/Library/Application Support/Cursor/User/settings.json` | JSON patch + 同目录 `.bak.<batch>` 文件 |
| Claude Code | `~/.claude/settings.json` + 可选 env | JSON patch + env 写到独立 `clawheart.env` shell 启动文件 |
| Codex CLI | env `OPENAI_API_BASE` + `OPENAI_API_KEY` | 写入 `~/.config/clawheart/env-codex.sh`，并提示用户 source |
| Cline | VS Code workspaceState（SQLite）/ user settings | 文件级备份 SQLite + JSON patch |
| Continue.dev | `~/.continue/config.json` | JSON patch（结构化路径） |
| Aider | `~/.aider.conf.yml` + 命令行环境 | YAML patch |
| Windsurf | `~/.config/windsurf/settings.json` | JSON patch |
| OpenClaw | `~/.openclaw/*` 自有 | JSON patch（自家产品最易接管） |

**关键设计**：所有 patch 都是**结构化操作**（JSON Path / YAML Path / env var name），不做正则 / 字符串替换。这样回滚精确。

---

## 4. UX 设计

### 4.1 入口（多入口，主入口在监控模式工具页 tier1 卡片）

```
┌─────── 监控模式 · tier1 卡片配置面板 ──────────────┐
│ 反向代理端点    http://127.0.0.1:19112/v1          │
│ 监听端口        127.0.0.1:[19112]    [保存]        │
│ 协议兼容路径    ☑ OpenAI Chat                       │
│                 ☑ Anthropic Messages                │
│                 ☐ Gemini                            │
│                 ...                                  │
│ ─────────────────────────────────────────────────  │
│ ⚡ 自动应用到已发现 Agent                            │
│ [打开中转配置助手 →]                                 │
└────────────────────────────────────────────────────┘
```

### 4.2 独立 L1 工具卡片

替换现有 "中转站 SOON v2.1" 占位 → 实装为「**Provider 中转配置**」工具：

```
src/components/grid/tools.config.ts:
  id: "relay" → 改名 "providers"
  label: "中转配置"
  description: "{n} Profile · {m} Agent 已接管"
  status: active
```

### 4.3 一键覆盖向导（L2 + 1 层 Dialog）

```
Step 1: 选择 Profile（或新建）
─────────────────────────────────────────────
┌─────────────────────────────────────────┐
│ 选择要应用的 Provider Profile             │
├─────────────────────────────────────────┤
│ ⊙ OpenRouter 主账号  (sk-or-***)         │
│ ○ Azure 公司账号     (azure-***)         │
│ ○ 自托管 LiteLLM     (lite-***)          │
│ ─────────────────────────                │
│ [+ 新建 Profile]                          │
└─────────────────────────────────────────┘


Step 2: 选择要接管的 Agent（默认全选已发现）
─────────────────────────────────────────────
┌─────────────────────────────────────────────────────┐
│  Agent       当前 base_url         风险      ☑       │
├─────────────────────────────────────────────────────┤
│  Cursor      api.openai.com         Safe     ☑      │
│  Claude Code api.anthropic.com      Safe     ☑      │
│  Codex CLI   (env: OPENAI_BASE)     Caution  ☑      │
│  Cline       (workspaceState)       Safe     ☑      │
│  Continue    api.openai.com         Safe     ☑      │
└─────────────────────────────────────────────────────┘


Step 3: Diff 预览（最关键，避免误改）
─────────────────────────────────────────────
┌─────────────────────────────────────────────────────────────┐
│  Cursor · settings.json                                      │
│  - cursor.openaiBaseUrl: "https://api.openai.com/v1"         │
│  + cursor.openaiBaseUrl: "http://127.0.0.1:19112/v1"         │
│  - cursor.openaiApiKey:  "sk-proj-xxxxxx" (来自当前配置)     │
│  + cursor.openaiApiKey:  "sk-claw-xxxxxx" (虚拟 key)         │
│                                                              │
│  Claude Code · ~/.claude/settings.json                       │
│  - ANTHROPIC_BASE_URL:   "https://api.anthropic.com"         │
│  + ANTHROPIC_BASE_URL:   "http://127.0.0.1:19112"            │
│  ...                                                          │
└─────────────────────────────────────────────────────────────┘
  [取消]                       [应用 5 处变更 →]


Step 4: 应用 + 结果
─────────────────────────────────────────────
✓ Cursor      已写入 settings.json     ← 备份 ID: snap-7c2
✓ Claude Code 已写入 settings.json     ← 备份 ID: snap-7c3
⚠ Codex CLI   环境变量已写入 ~/.zshrc，请重启 terminal 生效
✓ Cline       已写入 workspaceState    ← 备份 ID: snap-7c5
✓ Continue    已写入 config.json       ← 备份 ID: snap-7c6

  [完成]   [查看历史快照]   [一键回滚此批次]
```

### 4.4 历史快照管理（次级页面）

```
┌─── 已应用的批次 ────────────────────────────────┐
│ 2026-05-19 14:32  OpenRouter → 5 Agent  [回滚]│
│ 2026-05-18 09:15  Azure      → 3 Agent  [回滚]│
│ 2026-05-15 18:42  OpenRouter → 5 Agent  (已被 14:32 覆盖) │
└──────────────────────────────────────────────┘
```

---

## 5. 安全设计

### 5.1 凭据存储（Keychain）

- **永远不在 SQLite 存明文 API key**
- macOS: `keytar` → Keychain
- Windows: `keytar` → Credential Manager
- Linux: `libsecret` → GNOME Keyring / KWallet（fallback: `age` 加密文件 + 显式提示用户）

```rust
// 写入
keytar::set_password("clawheart-providers", &profile.id, &api_key)?;
profile.credential_ref = format!("keychain:clawheart-providers/{}", profile.id);

// 读取（仅在 proxy 转发请求时解密，永不暴露给前端）
let key = keytar::get_password("clawheart-providers", &profile.id)?;
```

### 5.2 虚拟 Key 路由

Agent 永远只看到 `sk-claw-xxxxxxx` 虚拟 key；ClawHeart 内部在 hudsucker / fetch_server handler 中做映射：

```rust
async fn forward_request(req: Request) -> Response {
    let virtual_key = extract_auth_token(&req);  // "sk-claw-xxx"
    let profile = profile_registry.lookup(virtual_key)?;
    let real_key = keychain::get(&profile.credential_ref)?;
    
    let mut upstream_req = req.clone();
    upstream_req.set_uri(profile.base_url + req.uri().path());
    upstream_req.set_auth(real_key);
    
    upstream_client.send(upstream_req).await
}
```

**好处**：
- 用户更换中转站 → 只改 ClawHeart 内 Profile，Agent 配置不动
- 虚拟 key 泄露 → 重新生成，真实 key 不变
- 真实 key 永远不离开本机

### 5.3 配置写入审计

每次 `apply()` 后写一条 receipt（复用 [`borrow-from-pipelock.md`](./borrow-from-pipelock.md) §2 的 Ed25519 链）：

```json
{
  "action": "agent_config_overwrite",
  "agent_id": "cursor-9c2",
  "batch_id": "batch-7c2",
  "before_hash": "sha256:abc...",
  "after_hash":  "sha256:def...",
  "snapshot_id": "snap-7c2",
  "signature":   "ed25519:..."
}
```

### 5.4 权限边界

- `ConfigProbe.apply()` 默认仅写入**用户主目录**下的文件
- 写入受 MDM 或 sudo 保护的位置时 → 升级为「手动复制 + 引导用户操作」流程（不强行 sudo）
- 对未知 Agent 平台 → 不应用，给出"手动接入向导"

### 5.5 失败原子性

每一个 Agent 的覆盖是**独立事务**：

- 单个失败不影响其他 Agent 已成功的覆盖
- 失败的 Agent 留下 `pending_rollback` 标记
- 批次结果汇总展示成功 / 失败 / 警告三类

---

## 6. IPC 契约（新增）

```rust
// src-tauri/src/commands/providers.rs

// === Profile CRUD ===
list_provider_profiles() -> Vec<ProviderProfile>
create_provider_profile(input: CreateProfileInput) -> ProviderProfile
update_provider_profile(id: String, patch: UpdateProfilePatch) -> ProviderProfile
delete_provider_profile(id: String) -> bool
set_default_provider_profile(id: String) -> ProviderProfile

// === 凭据（永不返回明文）===
set_provider_credential(profile_id: String, api_key: String) -> bool
test_provider_connection(profile_id: String) -> ConnTestResult

// === 一键覆盖向导 ===
scan_agent_configs() -> Vec<ProbeResult>           // 探测当前 Agent 配置
plan_overwrite(profile_id: String, agent_ids: Vec<String>) -> Vec<ConfigPatch>
apply_overwrite(patches: Vec<ConfigPatch>) -> ApplyResult   // 含 batch_id

// === 历史与回滚 ===
list_apply_batches() -> Vec<ApplyBatch>
list_snapshots(batch_id: Option<String>) -> Vec<AgentConfigSnapshot>
rollback_batch(batch_id: String) -> RollbackResult
rollback_snapshot(snapshot_id: String) -> RollbackResult

// === 运行时路由 ===
list_active_routings() -> Vec<ActiveRouting>
set_agent_routing(agent_id: String, profile_id: Option<String>) -> ActiveRouting
```

---

## 7. 实施路线（W6 + W7 两阶段）

### Phase 1（W6 · 集中 Profile + 手动接入）

- [ ] 新增 SQLite schema：`provider_profiles` / `agent_config_snapshots` / `active_routings`
- [ ] 实现 Profile CRUD + Keychain 凭据存储
- [ ] 新增 L1 工具卡片「中转配置」（替换 relay 占位）
- [ ] L2 主页：左侧 Profile 列表 + 右侧编辑器
- [ ] 虚拟 key 在 proxy 转发时映射（W5 hudsucker spike 同步接入）
- [ ] 文档化"手动接入向导"（每种 Agent 一份）

### Phase 2（W7 · Probe + 一键覆盖）

- [ ] 实现 ConfigProbe trait
- [ ] 优先实现 5 个 Probe：Cursor / Claude Code / Continue / Cline / OpenClaw
- [ ] 实现 plan_overwrite + diff 计算
- [ ] L2 增加"一键覆盖向导" Dialog（4 步式）
- [ ] 实现 snapshot 创建 + rollback
- [ ] 监控模式 tier1 卡片加"自动应用到已发现 Agent"入口

### Phase 3（W8+ · 扩展平台 + 增强）

- [ ] Codex / Aider / Windsurf Probe
- [ ] env var 写入（shell rc 检测与追加）
- [ ] 连接测试 + 性能基准
- [ ] 多账号 / 多 Profile 并存的 per-Agent 路由
- [ ] CLI 子命令 `clawheart providers ...`（管理员脚本化）

---

## 8. 与 ClawHeart 已有模块的整合点

| 模块 | 整合方式 |
|------|---------|
| `agents/` | 复用 6 平台 Agent 发现结果；ConfigProbe 与 PlatformScanner 一一对应 |
| `proxy/handler.rs` | 转发时检查 Authorization 中的虚拟 key → 路由到 Profile |
| `security/redact.rs` | 真实 API key 命中 DLP 时，确认是否为已知 Profile 凭据 → 区分误报 |
| `audit/receipts` | 每次 apply/rollback 触发签名 receipt |
| `commands/access_mode.rs` | tier1 协议适配器与 Profile.protocol 联动校验 |
| 设置 → 代理 | 增加"中转配置"快捷入口（与"监控模式"并列） |

---

## 9. 反向兼容 / 数据迁移

- **首次启动**：扫描已有 env 变量 / 配置文件 → 推断当前用户使用的中转站 → 自动建议创建 Profile
- **v1 → v2 迁移**（已有 `migration/v1-to-v2.md`）：v1 用户的 API Key 字段迁移到 Keychain + 自动创建 Profile

---

## 10. 不在本方案内的事项

明确排除：

| 不做 | 理由 |
|------|------|
| 自动发现 / 推荐中转站 | 不卖中转服务；不引入推荐偏向 |
| 跨设备 Profile 同步 | 凭据跨设备同步本质风险；用户应手动管理或使用企业策略 |
| 计费聚合 / token 统计跨 Profile 汇总 | budget 模块已有 token 统计，按 Profile 维度拓展即可，不在本方案范畴 |
| 浏览器 / 移动端工具的接管 | 本方案聚焦本机 Native / 终端工具 |
| Agent 工具配置 GUI 化 | 仅做 base_url + api_key 的接管，不替代用户在原工具内的其他配置 |

---

## 11. 风险与缓解

| 风险 | 缓解 |
|------|------|
| 用户的 Agent 配置文件格式不固定（版本升级会变） | Probe 内置 schema 版本检测；遇到未知版本退化为"提示用户手动接入" |
| 写入第三方应用的 settings.json 被该应用覆盖 | 改后立即调 `sha256` 校验；若 1 分钟内被改回则告警 |
| MDM 管控终端拒绝写入 | 探测时检测 EACCES，标记为 `Risky`，引导用户手动操作 |
| 虚拟 key 泄露后被用于 ClawHeart 仿冒 | 虚拟 key 仅在 127.0.0.1 监听上有效；带 origin 验证；可随时重置 |
| Profile 数量太多导致路由查询性能下降 | SQLite 索引 + 内存缓存（DashMap）；千级 Profile 内 O(1) |
| 用户误删 Profile 后真实 key 也丢失 | 删除前二次确认；30 天软删除窗口（仅 Profile 元数据，凭据立即从 Keychain 移除） |

---

## 12. 与 pipelock / LiteLLM 的对比

| 维度 | pipelock | LiteLLM Proxy | ClawHeart 本方案 |
|------|---------|---------------|------------------|
| 凭据集中 | 仅审计，不管 key | ✅ 虚拟 key 系统 | ✅ Keychain + 虚拟 key |
| 自动接管 Agent 配置 | ❌ | ❌ | ✅ ConfigProbe + 一键覆盖 |
| 桌面 GUI | ❌ CLI 优先 | 有限（Web 控制台） | ✅ Tauri 原生 + 4 步式向导 |
| 多 Profile 切换 | ❌ | ✅ | ✅ |
| 回滚机制 | ❌ | ❌ | ✅ snapshot + batch rollback |
| Diff 预览确认 | ❌ | ❌ | ✅ |

**结论**：本方案的差异化在于「**Probe + Snapshot + 一键 + 可逆**」四要素，是 ClawHeart 桌面端独占的 UX 优势。

---

## 13. 待用户决策的开放问题

| 问题 | 候选方案 |
|------|---------|
| 「中转配置」是独立 L1 工具，还是嵌入到「监控模式」？ | **建议独立**，避免监控模式工具页过大；监控模式 tier1 卡片放跳转入口 |
| Profile 是否支持模型映射（Profile A 仅提供 gpt-4o，Profile B 仅 claude）？ | v1 不做；user → "我两个模型分别用不同中转站" 比较罕见 |
| 是否支持团队共享 Profile？ | v2.2 企业策略模块再处理 |
| 失败的覆盖是否自动重试？ | 不重试；显式呈现失败原因让用户决策 |
| 是否支持"接管完成后定期 verify 配置仍指向 ClawHeart"？ | 推荐做；每日 1 次轻量探测，发现被改回则桌面通知 |

---

## 14. 总结

本方案以**集中凭据 + Probe 自动接管 + 完整可回滚**三个支柱解决用户在多 Agent / 多中转站场景下的配置碎片化问题。技术上重用 v2 已有的 6 平台 Agent 发现、Keychain 凭据、虚拟 key 思路，无需引入新依赖。

UX 上以四步向导（Profile → Agent → Diff → Apply）将复杂操作折叠为"两次点击即完成"，并把回滚作为一等公民暴露给用户，确保操作可逆。

实施可分两个 sprint（W6 集中 Profile + W7 Probe & 一键），与 hudsucker 真实激活同步上线，使 ClawHeart 在「桌面 Agent 配置治理」生态位上拉开与 pipelock / LiteLLM 的距离。
