# W11：MCP 协议接管

> 关联：[`agent-config-autoapply.md`](./agent-config-autoapply.md)、[`borrow-from-pipelock.md`](./borrow-from-pipelock.md) §7
> 状态：设计阶段（pending review）· 拟 W11 实施
> 前置依赖：W6/W7 Provider 体系 ✅、W10 credential_router ✅、`security/mcp_chains.rs` 10 条攻击链 ✅

---

## 0. 背景：MCP 是 Agent 安全的第二战场

MCP (Model Context Protocol，Anthropic 提出) 是 Claude Desktop / Cursor / Claude Code / Continue 等让 AI 调用本地工具（读文件、跑 shell、调网络）的标准协议。

**当前 ClawHeart 监控了什么**：
- ✅ LLM API 流量（通过反向代理 / hudsucker）
- ❌ AI → 工具的调用 = **完全盲区**

**为什么 MCP 是关键防线**：
- Agent 越来越自主：单次对话往往 3-10 次工具调用
- 攻击者目标已从「让 AI 说坏话」转向「让 AI 执行坏动作」
- v2 已经有了完整的安全检测能力（`mcp_chains` 10 条攻击链 + `mcp_baseline` 漂移检测），缺的是**真正的 MCP 代理拦截层**

---

## 1. MCP 三种传输模式与拦截策略

```
┌─ stdio 模式（最常见）────────────────────────────────────┐
│  Agent ←→ MCP Server 通过子进程 stdin/stdout 交换       │
│  例：Claude Desktop 启动 npx @modelcontextprotocol/...  │
│  拦截策略：proxy_command —— 让 Agent 启动 ClawHeart 的   │
│           proxy_mcp 子命令而非 MCP server 本身           │
└─────────────────────────────────────────────────────────┘

┌─ HTTP+SSE 模式（远程 MCP）──────────────────────────────┐
│  Agent ←→ MCP Server 通过 HTTP POST + SSE              │
│  例：远程 MCP 服务器（云端工具）                         │
│  拦截策略：作为 fetch_server 一个路径 (/mcp) 拦截        │
└─────────────────────────────────────────────────────────┘

┌─ HTTP-only 模式（W11 末期）────────────────────────────┐
│  纯请求-响应；MCP 0.1+ 规范                             │
│  拦截策略：与 HTTP+SSE 一致                             │
└─────────────────────────────────────────────────────────┘
```

---

## 2. 架构：MCP 拦截代理

### 2.1 stdio 模式（核心）

```
v1：                                  v2 W11：
─────────────────────────             ─────────────────────────
Claude Desktop                        Claude Desktop
   │ spawn                              │ spawn
   ▼                                    ▼
npx @mcp/server-filesystem           clawheart mcp-proxy \
   │                                    --upstream "npx @mcp/server-filesystem"
   │ stdio                              │
   ▼                                    │ 子进程 spawn upstream
本地文件系统                            ▼
                                     npx @mcp/server-filesystem
                                        │
                                        ▼
                                     本地文件系统

ClawHeart 工作流程：
1. Claude Desktop 通过 stdin 发 JSON-RPC 请求给 clawheart mcp-proxy
2. clawheart mcp-proxy 跑安全检查（input 方向：DLP / 工具白名单 / 攻击链记录）
3. 通过 clawheart mcp-proxy 内部的子进程 stdin 转发给真正的 MCP server
4. MCP server 返回结果给 clawheart mcp-proxy stdout
5. clawheart mcp-proxy 跑安全检查（output 方向：injection / 基线漂移）
6. 通过 stdout 返回给 Claude Desktop
```

### 2.2 HTTP+SSE 模式

挂在 fetch_server (`:19112`) 同一个 axum router：

```
POST /mcp                        Agent → ClawHeart
  JSON-RPC body                  例：tools/call { name: "fs.read", args: { path: "..." } }
        ▼
    安全检查（client → server 方向）
        ▼
    反查 virtual_key（如该 MCP 走 Profile）
        ▼
    转发到真实 MCP server (HTTP POST 或 SSE)
        ▼
    安全检查（server → client 方向）
        ▼
    流式回传给 Agent
```

---

## 3. 安全检查管线（已有 + 待接通）

| 检查项 | 已在 `security/` 中实现 | W11 接入位置 |
|--------|----------------------|------------|
| 工具基线冻结（sha256） | ✅ `mcp_baseline.rs` | client→server tools/list 时记录；漂移告警 |
| 攻击链 10 条侦察 | ✅ `mcp_chains.rs` ChainDetector | 每次 tools/call 调 observe(session, tool_name) |
| 参数 DLP | ✅ `redact.rs` 48 类正则 | tools/call params 入站时扫描 |
| 工具描述投毒 | ✅ `mcp.rs` SkillGuard 可复用 | tools/list 返回时扫工具描述 |
| 6-pass 归一化 | ✅ `normalizer.rs` | server→client 文本结果反注入 |
| 失败关闭 | ✅ `kill_switch.rs` | 任一层拒绝 → block + 自适应响应 |

**关键 trait（待写）**：

```rust
// src-tauri/src/proxy/mcp_proxy.rs
pub trait McpInterceptor: Send + Sync {
    /// client → server：扫工具参数
    fn check_request(&self, req: &JsonRpcRequest) -> McpVerdict;

    /// server → client：扫工具结果 + 漂移
    fn check_response(&self, req: &JsonRpcRequest, resp: &JsonRpcResponse) -> McpVerdict;
}

pub enum McpVerdict {
    Allow,
    Block { reason: String, layer: String },
    Warn { reason: String },
    Strip { sanitized_value: String },
}
```

---

## 4. CLI 接入：`clawheart mcp-proxy`

新增 Tauri binary 子命令（或独立 binary）：

```bash
# Claude Desktop 配置文件 ~/Library/Application Support/Claude/claude_desktop_config.json
{
  "mcpServers": {
    "filesystem": {
      "command": "clawheart",
      "args": ["mcp-proxy", "--upstream", "npx", "-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    }
  }
}
```

ClawHeart 接管流程：
1. Claude Desktop 启动 `clawheart mcp-proxy ...`
2. ClawHeart 进程 spawn 真实 upstream MCP server (`npx ...`)
3. 中间架设双向 JSON-RPC 拦截层
4. 所有拦截事件写入主进程 IPC（intercept_events 表）→ UI 监控页可见

**关键设计**：
- `clawheart mcp-proxy` 是**独立轻量子进程**，不依赖主 GUI 是否在运行
- 通过本地 socket / named pipe 与主进程通信上报事件
- 主 GUI 离线时 mcp-proxy 仍跑（fail-open 策略可配）

---

## 5. 一键接管 MCP（继承 W7 模式）

仿照 W7 一键覆盖向导：

```
中转配置工具页
└─ MCP Servers tab（W11 新增）
   ├─ 扫描已发现 Agent 的 MCP 配置（Claude Desktop / Cursor / Continue）
   ├─ 显示候选：
   │    Claude Desktop → filesystem MCP
   │      原始命令：npx @mcp/server-filesystem /tmp
   │      改写后：clawheart mcp-proxy --upstream "npx @mcp/server-filesystem /tmp"
   │      [Diff 预览]
   └─ 一键应用 + snapshot + 回滚
```

ConfigProbe trait 复用，新增 `McpConfigProbe`：

```rust
pub trait McpConfigProbe {
    fn inspect_mcp_servers(&self, agent: &DiscoveredAgent) -> Vec<McpServerEntry>;
    fn plan_mcp_takeover(&self, server: &McpServerEntry) -> McpConfigPatch;
}
```

为每个支持的 Agent 实现：
- Claude Desktop: `claude_desktop_config.json` 的 `mcpServers` 字段
- Cursor: `.cursor/mcp.json`
- Continue: `~/.continue/config.json` 的 `mcpServers` 段

---

## 6. UI 视图（嵌入 ProvidersTool）

```
ProvidersTool 顶部 tab 由 2 个变 3 个：
┌──────────────────────────────────────────────┐
│ [Profile 仓库] [历史批次] [MCP 接管 ⭐W11]    │
└──────────────────────────────────────────────┘

MCP 接管视图：
┌──────────────────────────────────────────────────────────┐
│  🔌 已发现 MCP Servers                                    │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Claude Desktop                                  │   │
│  │  ├─ filesystem (npx @mcp/server-filesystem)     │   │
│  │  │   状态：[未接管]   [一键接管 →]               │   │
│  │  ├─ github (npx @mcp/server-github)             │   │
│  │  │   状态：[已接管 ✓ 2026-05-19 14:32]          │   │
│  │  │   工具调用：147 次 · 阻断 3 次                 │   │
│  │  │   [查看事件] [回滚]                           │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
│  💡 已接管的 MCP server 会经过 ClawHeart 双向扫描：      │
│     - 工具参数中的密钥泄露                                │
│     - 工具结果中的 prompt injection                       │
│     - 10 条攻击链滑动窗口检测                              │
└──────────────────────────────────────────────────────────┘
```

---

## 7. 安全管线接通点

W11 实施时需要在 `mcp_proxy::McpInterceptor::check_*` 里调用：

```rust
// 入站（client → server）
fn check_request(&self, req: &JsonRpcRequest) -> McpVerdict {
    // 1. KillSwitch
    if self.app.kill_switch.snapshot() {
        return McpVerdict::Block { reason: "kill_switch".into(), layer: "L7".into() };
    }

    // 2. 攻击链记录（不阻断，只记录序列）
    if let Some(tool_name) = req.params.get("name").and_then(|v| v.as_str()) {
        let session_id = req.context.session_id.clone();
        let mut detector = self.app.chain_detector.lock().unwrap();
        if let Some(hit) = detector.observe(&session_id, tool_name) {
            // 命中攻击链 → 用 Verdict::Block
            return McpVerdict::Block {
                reason: format!("攻击链 {} 命中：{}", hit.id, hit.description),
                layer: "L3.MCP_CHAIN".into(),
            };
        }
    }

    // 3. 参数 DLP（防外泄）
    let args = serde_json::to_string(&req.params).unwrap_or_default();
    if let Some(hit) = self.app.dlp_scanner.scan(&args) {
        return McpVerdict::Block {
            reason: format!("DLP 命中：{}", hit.pattern),
            layer: "L2.DLP".into(),
        };
    }

    // 4. 工具白名单 / 黑名单
    // ...

    McpVerdict::Allow
}

// 出站（server → client）
fn check_response(&self, req: &JsonRpcRequest, resp: &JsonRpcResponse) -> McpVerdict {
    // 1. 工具描述漂移（tools/list 返回时）
    if req.method == "tools/list" {
        if let Some(drift) = self.baseline_check(&resp.result) {
            return McpVerdict::Warn { reason: drift };
        }
    }

    // 2. 工具结果中的 prompt injection
    if req.method == "tools/call" {
        let text = extract_text_content(&resp.result);
        let normalized = self.normalizer.normalize_six_pass(&text);
        if let Some(hit) = self.injection_scanner.scan(&normalized) {
            return McpVerdict::Strip {
                sanitized_value: redact_injection(&text, &hit),
            };
        }
    }

    McpVerdict::Allow
}
```

---

## 8. 实施路线

```
W11.1（3 天）：MCP 协议骨架
  - 新增 src-tauri/src/proxy/mcp_proxy.rs
  - 定义 McpInterceptor trait + JsonRpcRequest/Response 类型
  - 实现 stdio 双向转发器（不接安全检查）

W11.2（2 天）：CLI 子命令
  - clap 子命令 mcp-proxy --upstream <cmd> [args...]
  - spawn upstream 子进程 + 转发 stdio
  - 通过本地 socket 与主 GUI 通信上报

W11.3（2 天）：安全检查接入
  - check_request：KillSwitch + chain_detector + DLP
  - check_response：baseline drift + injection
  - intercept_events 写入

W11.4（3 天）：HTTP+SSE 模式
  - 在 fetch_server 加 POST /mcp 路由
  - axum 异步流式转发

W11.5（3 天）：一键接管 UI
  - McpConfigProbe trait + 3 实现（Claude Desktop / Cursor / Continue）
  - ProvidersTool 加 MCP 接管 tab
  - 接 W7 OverwriteWizard 同款流程

W11.6（2 天）：可视化与告警
  - 工具调用计数、阻断率
  - 攻击链命中详情展开
```

---

## 9. 不在 W11 范围

- ❌ MCP server 仿真（不做"假 MCP"）
- ❌ MCP 协议改造（保持 JSON-RPC 兼容）
- ❌ 远程 MCP server 发现（W17 云端策略再做）
- ❌ MCP server 性能基准（pipelock 类压测）

---

## 10. 风险与缓解

| 风险 | 缓解 |
|------|------|
| MCP 协议演进快，stdio 格式可能改变 | 用官方 `@modelcontextprotocol/sdk` 的 schema 类型；版本协商时降级到 raw 转发 |
| Agent 没启动主 GUI 时 mcp-proxy 失去事件出口 | 落地到本地 SQLite 缓冲；主 GUI 启动后批量上传 |
| 拦截耗时影响 Agent 体验 | 所有检查总和 ≤ 5ms；DLP / chain 使用预编译 trie；不引入网络 I/O |
| 已接管 MCP 后用户卸载 ClawHeart | snapshot 完整 + 回滚 → 还原原始 mcpServers 配置 |
| 复杂工具调用结构（多层嵌套） | JSON-RPC params 递归扫描；测试覆盖 fs / shell / web / db 四大类典型工具 |

---

## 11. 与 v2 已有模块的整合

| 模块 | 整合方式 |
|------|---------|
| `security/mcp.rs` | 提供 MCPInterceptor base，W11 继承扩展 |
| `security/mcp_chains.rs` | ChainDetector 在 check_request 内 observe |
| `security/mcp_baseline.rs` | check_response 在 tools/list 时调 freeze + 漂移比对 |
| `commands/intercept.rs` | 所有 McpVerdict::Block/Warn 写入 intercept_events |
| `agents/probes/*` | 各平台 ConfigProbe 扩展 inspect_mcp_servers |
| `commands/agent_config.rs` | 一键覆盖接 McpConfigProbe |

---

## 12. 衡量成功

- 可成功接管 Claude Desktop 至少 3 个流行 MCP server（filesystem / github / brave-search）
- 攻击链 SSH_KEY_EXFIL 在模拟攻击下 100% 命中 + block
- 单次工具调用 round-trip 延迟增量 < 5ms
- 用户开启 MCP 接管后能在 UI 看到工具调用频次与拦截事件
