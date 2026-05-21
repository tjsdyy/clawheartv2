# IPC API

> 完整 Tauri IPC 命令清单。所有命令必须显式列入 `src-tauri/capabilities/default.json`。

## 调用约定

```ts
import { invoke } from "@tauri-apps/api/core";

const status = await invoke<StatusInfo>("get_status");
```

错误以 `{ code: string; message: string }` 形式抛出。

## 命令清单

### Status / Settings

| Command | Args | Returns | 用途 |
|---------|------|---------|------|
| `get_status` | — | `StatusInfo` | 启动屏 + status bar |
| `get_proxy_status` | — | `ProxyStatus` | 代理服务运行状态 |
| `get_settings` | — | `Settings` | 通用设置 |
| `save_settings` | `{ settings: Settings }` | `void` | 保存设置 |
| `set_theme` | `{ theme: string }` | `void` | 切换主题 |

### Authentication

| Command | Args | Returns |
|---------|------|---------|
| `login` | `{ args: { email, password } }` | `AuthResult` |
| `logout` | — | `void` |
| `refresh_token` | — | `boolean` |

### Proxy

| Command | Args | Returns |
|---------|------|---------|
| `proxy_pause` | — | `ProxyControlResult` |
| `proxy_resume` | — | `ProxyControlResult` |
| `proxy_get_ca_cert` | — | `string`（CA 路径） |
| `proxy_install_ca` | — | `boolean` |

### Tools matrix

| Command | Args | Returns |
|---------|------|---------|
| `list_tools` | — | `ToolDescriptor[]` |
| `list_recent_events` | — | `InterceptEventListItem[]` |
| `trigger_kill_switch` | `{ activate: boolean }` | `boolean` |

### Intercept / Logs

| Command | Args | Returns |
|---------|------|---------|
| `list_intercept_events` | `{ limit?, offset? }` | `InterceptEventListItem[]` |
| `get_intercept_event` | `{ id }` | `Value` |
| `list_request_logs` | `{ limit? }` | `Value[]` |
| `export_request_logs` | `{ format: "json"\|"csv"\|"stix"\|"sarif" }` | `string`（路径） |

### Skills

| Command | Args | Returns |
|---------|------|---------|
| `list_skills` | — | `SkillListItem[]` |
| `toggle_skill` | `{ slug, enabled }` | `void` |
| `set_skill_safety` | `{ slug, label }` | `void` |
| `scan_skill` | `{ slug }` | `Value` |
| `sync_skills` | — | `number` |

### Danger Commands

| Command | Args | Returns |
|---------|------|---------|
| `list_danger_commands` | — | `DangerRuleItem[]` |
| `toggle_danger_command` | `{ rule_id, enabled }` | `void` |
| `sync_danger_commands` | — | `number` |

### Budget / Usage

| Command | Args | Returns |
|---------|------|---------|
| `list_budget_rules` | — | `BudgetRuleItem[]` |
| `set_budget_rule` | `{ rule }` | `void` |
| `get_token_usage` | `{ days? }` | `TokenUsageDay[]` |

### Agents

| Command | Args | Returns |
|---------|------|---------|
| `list_agents` | — | `DiscoveredAgent[]` |
| `discover_agents_now` | — | `DiscoveredAgent[]` |
| `list_mcp_servers` | `{ agent_id? }` | `Value[]` |

### Scan (80 项审计)

| Command | Args | Returns |
|---------|------|---------|
| `get_scan_items` | — | `ScanItemGroup[]` |
| `start_scan_run` | `{ items: string[] }` | `ScanRunResult` |
| `list_scan_history` | — | `Value[]` |
| `get_scan_progress` | `{ run_id }` | `number`（0.0-1.0） |

### Advisory

| Command | Args | Returns |
|---------|------|---------|
| `list_advisories` | — | `AdvisoryListItem[]` |
| `acknowledge_advisory` | `{ id }` | `void` |
| `subscribe_feed` | `{ url }` | `void` |

### Kill Switch

| Command | Args | Returns |
|---------|------|---------|
| `kill_switch_activate` | — | `KillSwitchStatus` |
| `kill_switch_reset` | — | `KillSwitchStatus` |
| `kill_switch_status` | — | `KillSwitchStatus` |

## Events (推送)

由 backend 主动推到 frontend：

```ts
import { listen } from "@tauri-apps/api/event";

listen<InterceptEvent>("ch://event/intercept", (e) => {
  // 实时拦截事件，用于刷新托盘 + 系统通知
});

listen<DiscoveredAgent>("ch://event/agent-discovered", (e) => {
  // Agent 发现 worker 每 60s 一次
});

listen<ProxyStatusEvent>("ch://event/proxy-status", (e) => {
  // 代理运行状态变化
});
```

## 阶段对应表

| 命令 | 完整实现阶段 |
|------|------------|
| status / settings / kill_switch / tools | W1 ✅ |
| proxy_* / intercept_* / list_request_logs | W5–W8 |
| skill_* / danger_* / scan_* | W9–W12（部分 W1） |
| agents / advisory / sync_* | W17–W20 |
| token_verify_*（v2.1） | v2.1 |
