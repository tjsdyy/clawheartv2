-- ClawHeart Desktop v2 — SQLite Schema
-- Generated 2026-05-17 · Apply via `storage::migrations::ensure_v2`.

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- ============================================================
--  Meta
-- ============================================================
CREATE TABLE IF NOT EXISTS schema_migrations (
  version    INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL
);

-- ============================================================
--  Settings (non-sensitive only; secrets → OS Keychain)
-- ============================================================
CREATE TABLE IF NOT EXISTS settings (
  key        TEXT PRIMARY KEY,
  value      TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- ============================================================
--  Danger commands
-- ============================================================
CREATE TABLE IF NOT EXISTS danger_commands (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  rule_id         TEXT UNIQUE NOT NULL,
  pattern         TEXT NOT NULL,
  pattern_type    TEXT NOT NULL DEFAULT 'regex',    -- substring | regex | semantic
  evidence_fields TEXT,                              -- JSON array
  mitre_attack_id TEXT,
  enabled         INTEGER NOT NULL DEFAULT 1,
  source          TEXT NOT NULL DEFAULT 'builtin',  -- builtin | user | cloud
  created_at      TEXT NOT NULL,
  updated_at      TEXT NOT NULL
);

-- ============================================================
--  Skills (合并 v1 三表)
-- ============================================================
CREATE TABLE IF NOT EXISTS skills (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  slug          TEXT UNIQUE NOT NULL,
  name          TEXT NOT NULL,
  description   TEXT,
  version       TEXT,
  system_status TEXT NOT NULL DEFAULT 'available',  -- available | deprecated | removed
  user_enabled  INTEGER NOT NULL DEFAULT 1,
  safety_label  TEXT NOT NULL DEFAULT 'unaudited', -- safe | warn | disabled | unaudited
  scan_score    INTEGER NOT NULL DEFAULT 0,
  install_path  TEXT,
  metadata      TEXT,
  installed_at  TEXT NOT NULL,
  updated_at    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS skill_scan_results (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  skill_slug      TEXT NOT NULL,
  score           INTEGER NOT NULL,
  risk_level      TEXT NOT NULL,
  blocked         INTEGER NOT NULL DEFAULT 0,
  hard_triggers   TEXT,                              -- JSON
  findings        TEXT NOT NULL,                     -- JSON
  scanned_at      TEXT NOT NULL,
  FOREIGN KEY (skill_slug) REFERENCES skills(slug)
);

-- ============================================================
--  LLM mappings (v1 兼容)
-- ============================================================
CREATE TABLE IF NOT EXISTS llm_mappings (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  source_id   TEXT NOT NULL,
  target_host TEXT NOT NULL,
  target_path TEXT,
  format      TEXT NOT NULL DEFAULT 'openai',
  enabled     INTEGER NOT NULL DEFAULT 1
);

-- ============================================================
--  Request logs (取代 v1 llm_usage_cost_events)
-- ============================================================
CREATE TABLE IF NOT EXISTS request_logs (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp    TEXT NOT NULL,
  agent_id     TEXT,
  session_id   TEXT,
  format       TEXT NOT NULL,
  provider     TEXT,
  model        TEXT,
  endpoint     TEXT NOT NULL,
  method       TEXT NOT NULL DEFAULT 'POST',
  status_code  INTEGER NOT NULL,
  blocked      INTEGER NOT NULL DEFAULT 0,
  bytes_in     INTEGER NOT NULL DEFAULT 0,
  bytes_out    INTEGER NOT NULL DEFAULT 0,
  latency_ms   INTEGER NOT NULL DEFAULT 0,
  raw          TEXT                              -- 原始 JSON 片段（脱敏后）
);
CREATE INDEX IF NOT EXISTS idx_rl_ts ON request_logs(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_rl_agent ON request_logs(agent_id);

-- ============================================================
--  Token usage
-- ============================================================
CREATE TABLE IF NOT EXISTS token_usage (
  id               INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp        TEXT NOT NULL,
  request_log_id   INTEGER,
  agent_id         TEXT,
  provider         TEXT NOT NULL,
  model            TEXT NOT NULL,
  input_tokens     INTEGER NOT NULL DEFAULT 0,
  output_tokens    INTEGER NOT NULL DEFAULT 0,
  cache_read       INTEGER NOT NULL DEFAULT 0,
  cache_creation   INTEGER NOT NULL DEFAULT 0,
  cost_usd         REAL NOT NULL DEFAULT 0,
  FOREIGN KEY (request_log_id) REFERENCES request_logs(id) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_tu_ts ON token_usage(timestamp DESC);

-- ============================================================
--  Budget rules
-- ============================================================
CREATE TABLE IF NOT EXISTS budget_rules (
  id        INTEGER PRIMARY KEY AUTOINCREMENT,
  provider  TEXT NOT NULL,                       -- "global" | "anthropic" | ...
  model     TEXT,
  period    TEXT NOT NULL,                       -- daily | monthly
  limit_usd REAL NOT NULL,
  enabled   INTEGER NOT NULL DEFAULT 1
);

-- ============================================================
--  Intercept events
-- ============================================================
CREATE TABLE IF NOT EXISTS intercept_events (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp       TEXT NOT NULL,
  event_type      TEXT NOT NULL,
  severity        TEXT NOT NULL,
  signal_class    TEXT NOT NULL,
  rule_id         TEXT,
  mitre_attack_id TEXT,
  confidence      TEXT NOT NULL,
  details         TEXT NOT NULL,
  evidence        TEXT,
  prompt_snippet  TEXT,
  agent_id        TEXT,
  session_id      TEXT,
  cloud_id        INTEGER,
  created_at      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ie_ts ON intercept_events(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_ie_type ON intercept_events(event_type);

-- ============================================================
--  MCP baselines & chain windows
-- ============================================================
CREATE TABLE IF NOT EXISTS mcp_tool_baselines (
  id               INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id       TEXT NOT NULL,
  server_id        TEXT NOT NULL,
  tool_name        TEXT NOT NULL,
  description_hash TEXT NOT NULL,
  capability       TEXT,
  frozen_at        TEXT NOT NULL,
  UNIQUE(session_id, server_id, tool_name)
);

CREATE TABLE IF NOT EXISTS mcp_chain_window (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id   TEXT NOT NULL,
  tool_name    TEXT NOT NULL,
  observed_at  TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_mcw_session ON mcp_chain_window(session_id, observed_at);

-- ============================================================
--  Agents & drift
-- ============================================================
CREATE TABLE IF NOT EXISTS discovered_agents (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  platform     TEXT NOT NULL,
  agent_name   TEXT NOT NULL,
  config_path  TEXT,
  process_name TEXT,
  last_seen    TEXT NOT NULL,
  mcp_servers  TEXT,
  config_hash  TEXT,
  status       TEXT NOT NULL DEFAULT 'active',
  UNIQUE(platform, agent_name)
);

CREATE TABLE IF NOT EXISTS drift_baselines (
  path          TEXT PRIMARY KEY,
  sha256        TEXT NOT NULL,
  captured_at   TEXT NOT NULL,
  user_approved INTEGER NOT NULL DEFAULT 0
);

-- ============================================================
--  Advisories
-- ============================================================
CREATE TABLE IF NOT EXISTS security_advisories (
  id          TEXT PRIMARY KEY,
  severity    TEXT NOT NULL,
  title       TEXT NOT NULL,
  affected    TEXT NOT NULL,
  cvss_score  REAL,
  action      TEXT,
  published   TEXT NOT NULL,
  fetched_at  TEXT NOT NULL,
  dismissed   INTEGER NOT NULL DEFAULT 0
);

-- ============================================================
--  Conversation history (v1 兼容 + 强制脱敏)
-- ============================================================
CREATE TABLE IF NOT EXISTS conversation_history (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id  TEXT NOT NULL,
  agent_id    TEXT,
  role        TEXT NOT NULL,
  content     TEXT NOT NULL,                   -- 已脱敏的
  created_at  TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ch_session ON conversation_history(session_id, created_at);

-- ============================================================
--  Sync state (single table for all entities)
-- ============================================================
CREATE TABLE IF NOT EXISTS sync_state (
  entity       TEXT PRIMARY KEY,                 -- skills | danger | usage | intercept
  last_sync_at TEXT,
  cursor       TEXT,
  status       TEXT NOT NULL DEFAULT 'idle'
);

-- ============================================================
--  Scan history
-- ============================================================
CREATE TABLE IF NOT EXISTS scan_runs (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  started_at    TEXT NOT NULL,
  completed_at  TEXT,
  total_checks  INTEGER NOT NULL DEFAULT 0,
  passed        INTEGER NOT NULL DEFAULT 0,
  failed        INTEGER NOT NULL DEFAULT 0,
  warned        INTEGER NOT NULL DEFAULT 0,
  skipped       INTEGER NOT NULL DEFAULT 0,
  items_json    TEXT NOT NULL,                  -- 用户勾选的扫描项
  results_json  TEXT
);
CREATE INDEX IF NOT EXISTS idx_sr_started ON scan_runs(started_at DESC);

-- ============================================================
--  Provider profiles (第三方 LLM 中转 API 集中管理 · W6 接入)
--  真实 API key 仅存 OS Keychain，credential_ref 是 keychain key 名
-- ============================================================
CREATE TABLE IF NOT EXISTS provider_profiles (
  id              TEXT PRIMARY KEY,                 -- uuid v7
  name            TEXT NOT NULL,
  provider_kind   TEXT NOT NULL,                    -- openrouter | azure | deepbricks | newapi | openai | anthropic | litellm | custom
  protocol        TEXT NOT NULL DEFAULT 'openai',   -- openai | anthropic | gemini | ollama | openai_responses
  base_url        TEXT NOT NULL,
  credential_ref  TEXT NOT NULL,                    -- Keychain 项名（不存明文 key）
  default_model   TEXT,
  headers_json    TEXT,                              -- 附加请求头 (JSON object)
  virtual_key     TEXT NOT NULL UNIQUE,             -- sk-claw-xxx 对外发的虚拟 key
  is_default      INTEGER NOT NULL DEFAULT 0,
  enabled         INTEGER NOT NULL DEFAULT 1,
  created_at      TEXT NOT NULL,
  updated_at      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_pp_virtual_key ON provider_profiles(virtual_key);
CREATE INDEX IF NOT EXISTS idx_pp_default ON provider_profiles(is_default);

-- ============================================================
--  Agent config snapshots (W7 一键覆盖前快照，支持精确回滚)
-- ============================================================
CREATE TABLE IF NOT EXISTS agent_config_snapshots (
  id               TEXT PRIMARY KEY,                 -- uuid v7
  batch_id         TEXT NOT NULL,                    -- 同一次"一键覆盖"批次共享
  agent_id         TEXT NOT NULL,
  agent_platform   TEXT NOT NULL,                    -- cursor | claude_code | codex | continue | openclaw | ...
  config_path      TEXT NOT NULL,
  config_kind      TEXT NOT NULL,                    -- json_file | toml_file | env_var | vsx_setting
  before_value     TEXT NOT NULL,
  after_value      TEXT NOT NULL,
  applied_at       TEXT NOT NULL,
  rolled_back_at   TEXT,
  profile_id       TEXT,
  FOREIGN KEY (profile_id) REFERENCES provider_profiles(id) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_acs_batch ON agent_config_snapshots(batch_id);
CREATE INDEX IF NOT EXISTS idx_acs_agent ON agent_config_snapshots(agent_id);
CREATE INDEX IF NOT EXISTS idx_acs_applied ON agent_config_snapshots(applied_at DESC);

-- ============================================================
--  Active routings (运行时 Agent → Profile 路由表)
--  代理收到请求时以 virtual_key 反查 profile_id
-- ============================================================
CREATE TABLE IF NOT EXISTS active_routings (
  agent_id     TEXT PRIMARY KEY,
  profile_id   TEXT NOT NULL,
  virtual_key  TEXT NOT NULL,
  updated_at   TEXT NOT NULL,
  FOREIGN KEY (profile_id) REFERENCES provider_profiles(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_ar_vkey ON active_routings(virtual_key);

-- ============================================================
--  Agent ↔ Channel assignments (Agent 显式分配的渠道)
--  N:M 关系：一个渠道可分配给多个 Agent，一个 Agent 可有多个渠道
--  AgentsTool 只显示分配过来的渠道；模型渠道库管理 CRUD
-- ============================================================
CREATE TABLE IF NOT EXISTS agent_channel_assignments (
  agent_id     TEXT NOT NULL,                 -- "claude/Claude Code" 等 platform/agent_name 形式
  profile_id   TEXT NOT NULL,
  assigned_at  TEXT NOT NULL,
  PRIMARY KEY (agent_id, profile_id),
  FOREIGN KEY (profile_id) REFERENCES provider_profiles(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_aca_agent ON agent_channel_assignments(agent_id);
CREATE INDEX IF NOT EXISTS idx_aca_profile ON agent_channel_assignments(profile_id);

-- ============================================================
--  Security rule overrides (安全规则覆盖)
--  每条 builtin 规则的 enabled / action 覆盖；NULL = 使用默认
-- ============================================================
CREATE TABLE IF NOT EXISTS security_rule_overrides (
  rule_kind   TEXT NOT NULL,                  -- "danger" | "injection" | "credential" | "skill" | "audit"
  rule_id     TEXT NOT NULL,                  -- "DG-001" / "INJ-001" / "CL-OPENAI" / "SK-001" / "FP-001" ...
  enabled     INTEGER NOT NULL DEFAULT 1,
  action      TEXT,                            -- NULL = 用默认；"block" / "warn" / "skip"
  updated_at  TEXT NOT NULL,
  PRIMARY KEY (rule_kind, rule_id)
);
CREATE INDEX IF NOT EXISTS idx_sro_kind ON security_rule_overrides(rule_kind);

-- ============================================================
--  Skill backups (技能备份历史)
--  每次「技能备份」工具的打包记录
-- ============================================================
CREATE TABLE IF NOT EXISTS skill_backups (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  created_at    TEXT NOT NULL,                  -- ISO timestamp
  zip_path      TEXT NOT NULL,                  -- 输出 zip 的绝对路径
  skill_count   INTEGER NOT NULL,
  total_bytes   INTEGER NOT NULL,
  skill_ids     TEXT NOT NULL,                  -- JSON array of source-of-truth ids
  skill_names   TEXT NOT NULL,                  -- JSON array; UI 展示无需再回查
  zip_exists    INTEGER NOT NULL DEFAULT 1      -- 1 = 文件仍在；0 = 用户删除
);
CREATE INDEX IF NOT EXISTS idx_skill_backups_created ON skill_backups(created_at DESC);

-- Bootstrapping: register v2
INSERT OR IGNORE INTO schema_migrations(version, applied_at) VALUES (2, datetime('now'));
