# 安全规则配置菜单 · 设计稿

## 目标

把当前散布在代码常量中的安全规则（合计 130+ 条）做成 UI 可视化的手动开关。

## 规则盘点（来自真实代码）

| 类目 | 数量 | 代码 | 当前状态 |
|---|---|---|---|
| 危险指令 DangerRule | 17 | `security/danger.rs::BUILTIN_RULES` | 全部启用（无 enabled 字段） |
| 提示词注入 InjectionPattern | 25+ (5 类) | `security/injection.rs::PATTERNS` | 全部启用 |
| 凭据指纹 CredPattern | 12+ | `security/redact.rs::PATTERNS` | 全部启用 |
| SkillGuard Rule | 7 起步集 | `security/skill_scanner.rs::RULES` | 全部启用 |
| 80 项 AuditCheck | 80 (8 类) | `security/checks/*.rs` | 已有 user 勾选 UI（仅扫描时） |
| 预算规则 | 用户自定义 | `budget_rules` 表 | 已有完整 CRUD |
| 危险指令（DB 持久化版本） | 用户自定义 | `danger_commands` 表 | 已有 toggle UI |

**未实现的部分**：上述前 4 类规则当前是 `pub const` 编译期常量，没有 enabled 字段、没有 DB 持久化。要做配置菜单首先得加 DB 层 + 后端 IPC。

## 主推方案 · 左侧分类 + 右侧规则列表

```
┌────────────────────────────────────────────────────────────────────┐
│ ← 返回 · 设置                                            ⌘K  🎨   │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│ ┌─────────────────┬────────────────────────────────────────────┐  │
│ │ 通用            │  搜索 [_______________] 🔍   ☐ 仅显示已禁用 │  │
│ │ 主题            │ ─────────────────────────────────────────── │  │
│ │ 监控模式 ▸     │  危险指令 · 17 条 · 启用 17/17              │  │
│ │ ▸ 安全规则 ⌗   │                              [全选] [全部禁用]│  │
│ │ ───────────────│                                              │  │
│ │ ⚠ 危险指令 17  │  ☑ DG-001  [HardBlock]   rm -rf /            │  │
│ │ 🛡 提示词注入25│  ☑ DG-002  [HardBlock]   fork bomb           │  │
│ │ 🔐 凭据指纹 12 │  ☑ DG-003  [HardBlock]   curl ... \| bash    │  │
│ │ 🧬 技能规则 7  │  ☑ DG-005  [HardBlock]   mkfs /dev/...       │  │
│ │ 🔍 扫描 80     │  ☑ DG-013  [HardBlock]   iwr ... \| iex      │  │
│ │ ───────────────│  ...                                         │  │
│ │ 高级            │  ☐ DG-019  [Warn]   echo >> ~/.bashrc       │  │
│ │ ───────────────│                                              │  │
│ │ 危险操作        │  共 17 条 · 14 启用 · 3 禁用  · 最近 7 天命中42 │
│ └─────────────────┴────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────┘
```

每条规则行的展开样式（点行任意位置或「详情」按钮）：

```
☑ DG-003 [HardBlock]  curl <url> | bash
└─ Pattern (regex):  \bcurl\b[^|]*\|\s*(bash|sh|zsh|fish)
   命中场景:         curl https://example.com/install.sh | bash
   触发动作:         [HardBlock ▼]   ← 可降级为 Warn / Skip
   过去 30 天命中:   12 次（最近 2026-05-19 14:21）
   修复建议:         先 curl -o 落盘再审计内容后执行
   [测试…] [查看命中历史] [恢复默认]
```

### 状态徽章颜色映射

| 徽章 | 颜色 | 含义 |
|---|---|---|
| HardBlock | critical 红 | HardTrigger 命中直接 Block |
| Block | critical 红 | 命中直接拦截 |
| Warn | high 橙 | 命中只告警不拦截 |
| Weighted | accent 蓝 | 加权扣分类 |
| Skipped | text-muted 灰 | 当前禁用或未实现 |

### 关键交互

1. **左侧分类节点**显示「enabled/total」+ 图标
2. **顶部搜索框**模糊匹配 ID / description / pattern
3. **右上「仅显示已禁用」筛选**便于审计偏移默认配置
4. **全选 / 全部禁用** 批量操作，但 **HardBlock 类规则禁用需二次确认**
5. **触发动作可调** — 部分规则可从 HardBlock 降级 Warn（仅供高级用户）
6. **「恢复默认」**单条或整组重置
7. **「最近命中」** 从 `intercept_events` 表查（已有数据）
8. **底部条**展示当前类目的统计 + 命中次数

## 备选方案对比

| 方案 | 优点 | 缺点 |
|---|---|---|
| **A · 全规则平铺表格** | 一屏看到所有规则、便于对比 | 130+ 行密度过高、无法兼顾详情 |
| **B · 左侧分类 + 右侧列表（主推）** | 与现有 SettingsTool 一致、清晰分层 | 切换分类时上下文丢失 |
| **C · 树状大纲（手风琴）** | 单页所有内容、可全展开 | 大量规则全展开后过长 |
| **D · 命令面板风格（⌘K）** | 极致搜索体验 | 不适合做"逐条勾选"批量操作 |

**推荐 B**：与 SettingsTool 设计语言一致，复用左侧导航 aside 组件。

## 与现有架构的关系

### 现有可复用部分

- `SettingsTool.tsx` 左侧 aside · 沉浸式 chrome 已就位
- `danger_commands` 表已支持 toggle（但仅覆盖 DB 持久化的危险指令，不含 17 条 builtin）
- `intercept_events` 表已记录命中历史，可直接做"过去 N 天命中"统计

### 需要新增

1. **后端**
   - 新表 `security_rule_overrides`（rule_kind / rule_id / enabled / action_override）
   - `security/rule_registry.rs` 把 4 类常量规则注册为统一 `RuleDescriptor`
   - 4 个 IPC：`list_security_rules` / `toggle_security_rule` / `set_rule_action` / `reset_rule_defaults`
   - pipeline 内每条规则前检查 override（O(1) HashMap）

2. **前端**
   - `SecurityRulesPanel.tsx` 新组件
   - `useSecurityRules` hook
   - 接入 `SettingsTool` 作为新分类节点

3. **DB schema**
   ```sql
   CREATE TABLE security_rule_overrides (
     rule_kind   TEXT NOT NULL,     -- "danger" | "injection" | "credential" | "skill"
     rule_id     TEXT NOT NULL,     -- "DG-001" / "INJ-001" / ...
     enabled     INTEGER NOT NULL DEFAULT 1,
     action      TEXT,              -- NULL = 用默认；"block" / "warn" / "skip"
     updated_at  TEXT NOT NULL,
     PRIMARY KEY (rule_kind, rule_id)
   );
   ```

### 风险点

- **HardBlock 降级风险** — 用户可能被引导降级安全规则。需要在 UI 强标注 + 二次确认
- **fail-closed 兼容** — 即使规则禁用，KillSwitch 层依然兜底，不影响 L7
- **配置可导出/导入** — 团队场景下需要支持规则集 JSON 导出（v2.1）

## 关键决策待用户确认

下面 3 个问题影响实现策略。

1. **规则覆盖粒度**：仅「启用/禁用」二态，还是允许"降级触发动作"（Block → Warn → Skip）？
2. **80 项 AuditCheck 是否纳入**：当前 ScanTool 已有勾选 UI（仅扫描时），是否合并到统一规则配置菜单？
3. **是否支持规则集导出/导入**：JSON 格式，便于团队对齐配置。
