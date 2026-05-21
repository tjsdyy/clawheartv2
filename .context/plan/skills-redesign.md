# Skills 工具重设计 — UI + 数据模型方案

## 定位变更

| 维度 | 旧 v1（已实现） | 新 v2（本方案） |
|---|---|---|
| 定位 | 只读备份 + 鉴定 | **统一管理 + 备份 + 鉴定** |
| 写操作 | 只 zip 导出 | 引入 SSOT + symlink（参考 cc-switch） |
| 数据中心 | 各 Agent 目录散落 | `~/.clawheart/skills/` 单一事实源 |
| 跨 Agent | 各 Agent 独立看 | per-Agent toggle 共享同一份 |
| 同步 | 无 | **暂不实现**（明确不做） |

## 核心概念

### SSOT（Single Source of Truth）
- 默认位置：`~/.clawheart/skills/`（用户可改）
- 每个 skill 在 SSOT 中只有一份真实文件
- 各 Agent 目录（`~/.claude/skills/<id>`、`~/.openeva/skills/<id>` 等）放 symlink 指向 SSOT
- symlink 失败时回退为深拷贝（Windows 等场景）

### 三种 Skill 状态

| 状态 | 含义 | UI 视觉 |
|---|---|---|
| `Managed` | 在 SSOT 中，各 Agent 是 symlink 到它 | 绿色 ● SSOT 标签 |
| `Unmanaged` | 散落在 Agent 目录的真实文件，SSOT 没有 | ⚠ 散落 标签 + 「移入 SSOT」按钮 |
| `Orphan` | SSOT 中存在但所有 Agent 都未启用 | 灰色 ○ 未启用 标签 |

### per-Agent Binding

每个 Skill 有一个 `bindings` 字段，记录在每个已发现 Agent 下的状态：

```rust
enum BindingKind {
    None,                          // Agent 目录中没有
    Symlink { target: PathBuf },   // symlink → SSOT 或别处
    Real,                          // 真实目录（unmanaged）
    Broken,                        // symlink 但目标不存在
}
```

UI 把 `Symlink { target: ssot_path }` 显示为「启用」，其他状态显示为「未启用」或「错误」。

## ASCII 线框

### Tab 1 「技能库」主视图

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ 技能库 ▼  扫描报告  备份历史                                                  │
├──────────────┬───────────────────────────────────────────────────────────────┤
│ ▼ 概览        │ [🔍 搜索…………] [全选] [移入 SSOT] [扫描全部] [▼ 备份选中 (3)]    │
│   全部     24 ├───────────────────────────────────────────────────────────────┤
│   未管理   5  │ ☑ 📄 web-fetch  v1.2.0  [● SSOT]  Browser fetch with robots… │
│   仅 SSOT  3  │   [C ✓][O ✓][X _][G _][P _]   12 files · 24KB   评分 95  →   │
│ ▼ Agent       │ ───────────────────────────────────────────────────────────  │
│   .claude  18 │ ☐ 📄 postgres-mcp  v0.4.1  [● SSOT]  Read-only PostgreSQL    │
│   .openeva  9 │   [C _][O ✓][X _][G _][P _]   8 files · 18KB    评分 78  →  │
│   .codex   5  │ ───────────────────────────────────────────────────────────  │
│   .cursor  3  │ ☐ 📄 safe-runner  v2.0.0  [⚠ 散落 in .claude]                 │
│ ▼ 鉴定        │   [C ✓][O _][X _][G _][P _]   6 files · 12KB    未扫描   →  │
│   未扫描   18 │   ▸ 「移入 SSOT」让其他 Agent 也能共享                        │
│   ✓ 安全    4 │ ───────────────────────────────────────────────────────────  │
│   ⚠ 警告    1 │ ☐ 📄 legacy-mcp  v0.0.3  [● SSOT][○ 未启用]                  │
│   ✗ 危险    1 │   [C _][O _][X _][G _][P _]   3 files · 5KB     评分 18 ✗  → │
│              │   ▸ 「卸载」清除 SSOT 主体                                    │
└──────────────┴───────────────────────────────────────────────────────────────┘
```

**图例**：
- `[C ✓]` = claude 已启用（symlink 存在且指向 SSOT），高亮色
- `[C _]` = claude 未启用（点击启用 → 创建 symlink）
- `[C !]` = symlink 损坏或指向非 SSOT（红色警告）
- 字母用 Agent 首字母：C=claude, O=openeva, X=cursor (X 字形)…实际 UI 用 Agent 真实首字母 + 颜色 token

### 单 Skill 行交互

```
┌─────────────────────────────────────────────────────────────────────────┐
│ ☑  📄  web-fetch  v1.2.0  [● SSOT]                          评分 95   → │
│        Browser fetch with robots.txt compliance                          │
│        ┌──────────────────────────┐                                      │
│        │ [C ✓] [O ✓] [X _] [G _]  │  ← 点 [X _] 即在 cursor 启用         │
│        │  启用2  共享于 2 个 Agent  │                                      │
│        └──────────────────────────┘                                      │
│        12 files · 24KB · ~/.clawheart/skills/web-fetch                  │
└─────────────────────────────────────────────────────────────────────────┘
```

### 顶部工具条按钮

| 按钮 | 触发动作 | 启用条件 |
|---|---|---|
| 全选 | 选中当前过滤视图所有 skill | 列表非空 |
| 移入 SSOT | 把选中的 Unmanaged 全部迁入 SSOT | 至少有 1 个 Unmanaged 被选中 |
| 扫描全部 | 对**当前视图**所有 skill 跑 skill_scanner | 列表非空 |
| 备份选中 | 把选中的打包为 zip（沿用现有逻辑） | 至少选 1 个 |

### 详情抽屉（升级版）

复用现有 `SkillDetailDrawer`，新增「绑定 / 管理」面板：

```
┌─ web-fetch v1.2.0 ──────────────────────────────────── × ─┐
│ 路径：~/.clawheart/skills/web-fetch       [📋 复制路径]   │
│ contentHash: a3f8…b2e4  ●已计算                          │
├──────────────────────────────────────────────────────────┤
│ 文件树  │  SKILL.md ▼  README  绑定                        │
├─────────┼────────────────────────────────────────────────┤
│ SKILL.md │ ┌─ Agent 绑定 ──────────────────────────────┐  │
│ README.md│ │ [✓] .claude    symlink → SSOT             │  │
│ index.py │ │ [✓] .openeva   symlink → SSOT             │  │
│ examples/│ │ [ ] .cursor    点击启用                    │  │
│   …      │ │ [ ] .codex     点击启用                    │  │
│          │ └────────────────────────────────────────────┘  │
│          │ [⚠] 卸载 — 从 SSOT 删除主体 + 清除所有 symlink   │
└─────────┴────────────────────────────────────────────────┘
```

## 数据模型升级

### 后端 `DiscoveredSkill`（扩展）

```rust
pub struct DiscoveredSkill {
    // === 旧字段保留 ===
    pub id: String,                // 改为 slug 形式（不含 agent 前缀），跨 Agent 唯一
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub file_count: u32,
    pub total_bytes: u64,
    pub has_skill_md: bool,

    // === 新字段 ===
    pub content_hash: Option<String>,    // 借鉴 cc-switch；目录内容 SHA256（前 16 字符）
    pub in_ssot: bool,
    pub ssot_path: Option<String>,       // SSOT 中的完整路径
    pub bindings: Vec<AgentBinding>,     // 每个发现的 Agent 的绑定状态

    // === 废弃 ===
    // source_agent 用 bindings 计算（取第一个 enabled 的，或 SSOT 主目录）
    // source_path 改用 ssot_path 或 bindings[0].path
}

pub struct AgentBinding {
    pub agent_name: String,           // "claude" / "openeva" / "cursor" / ...
    pub agent_dir_exists: bool,       // ~/.<agent>/ 目录是否存在（决定是否显示这个 Agent）
    pub binding: BindingKind,
}

pub enum BindingKind {
    None,                                  // Agent 目录中找不到此 skill
    Real { path: PathBuf, modified: i64 }, // 实际文件（unmanaged）
    Symlink { path: PathBuf, target: PathBuf, points_to_ssot: bool },
    Broken { path: PathBuf, target: PathBuf },  // symlink 但目标不存在
}
```

### 新 IPC 命令

```rust
// 配置
get_ssot_config()                      -> SsotConfig { path, exists, total_skills }
set_ssot_path(new_path)                -> SsotConfig

// 写操作（所有都受 dry-run 保护，参考 W8 apply_real_enabled 风险开关）
move_skill_to_ssot(id)                 -> DiscoveredSkill   // 把 Unmanaged 迁入 SSOT
toggle_skill_binding(id, agent, on)    -> DiscoveredSkill   // 创建/移除某 Agent 的 symlink
uninstall_skill(id)                    -> ()                // 删 SSOT 主体 + 所有 symlink
repair_broken_binding(id, agent)       -> DiscoveredSkill   // 修复指向错误 / 损坏的 symlink

// 现有保留
discover_local_skills()                -> Vec<DiscoveredSkill>
get_local_skill_detail(id)             -> SkillDetail
scan_local_skill(id)                   -> ScanReport
backup_local_skills(ids, path?)        -> BackupResult
list/delete_skill_backups
```

## 写操作安全模型

参考 W8 `apply_real_enabled` 模式：

1. **默认 dry-run**：所有写操作（move_to_ssot / toggle / uninstall）默认写到沙箱 `~/.clawheart/dry-run/skills/`，**不动**真实目录
2. **真写需要二次确认**：Settings → 安全 中有 `skill_apply_real_enabled` 开关（首次启用弹风险确认 Dialog）
3. **每次危险操作前 UI 弹确认**：尤其是 `uninstall_skill`（删主体 + 所有 symlink，不可逆）
4. **自动备份**：迁入 SSOT 前自动 zip 一份原始目录到 `~/.clawheart/auto-backups/<ts>.zip`

## 发现算法（升级）

```rust
fn discover_all() -> Vec<DiscoveredSkill> {
    let ssot = ssot_path();
    let agents = list_agent_dirs();   // 扫描 ~/.<agent>/skills/ + ~/.cursor/extensions/

    let mut by_id: HashMap<String, DiscoveredSkill> = HashMap::new();

    // Pass 1: SSOT 中的所有 skill 先建条目
    for dir in fs::read_dir(ssot)? {
        let s = inspect_skill(&dir.path());
        s.in_ssot = true;
        s.ssot_path = Some(dir.path());
        by_id.insert(s.id.clone(), s);
    }

    // Pass 2: 扫各 Agent 目录，合并到对应 id
    for agent in agents {
        for dir in fs::read_dir(agent.skills_dir)? {
            let id = compute_id(&dir);
            let binding = classify_binding(&dir.path(), &ssot);
            by_id.entry(id).or_insert_with(|| inspect_skill(&dir.path()))
                .bindings.push(AgentBinding { agent_name: agent.name, ... });
        }
    }

    by_id.into_values().collect()
}
```

**id 计算**：优先 frontmatter `name`，否则目录名；做小写 + 连字符标准化，让 `~/.claude/skills/Foo` 与 `~/.openeva/skills/foo` 合并为一条。

## 前端组件拆分

```
SkillsBackupTool/
├── SkillsBackupTool.tsx        — 入口路由 (3 tabs)
├── DiscoverView.tsx            — Tab 1（左过滤 + 右列表）
│   ├── FilterSidebar.tsx       — 左侧 3 段分组
│   ├── SkillRow.tsx            — 单行，含 AgentToggleGroup
│   └── AgentToggleGroup.tsx    — 字母按钮组（C/O/X/G/P）
├── ScanReportsView.tsx         — Tab 2（保留）
├── BackupHistoryView.tsx       — Tab 3（保留）
├── SkillDetailDrawer.tsx       — 详情抽屉（升级 + Binding 面板）
└── MoveToSsotDialog.tsx        — 迁入 SSOT 确认弹窗
```

## 工作流示例

### 场景 A：首次启动
1. SSOT 目录不存在 → 创建 `~/.clawheart/skills/`
2. 扫描发现 5 个 unmanaged skill（散落在 .claude / .codex）
3. UI 默认筛选「未管理」，引导用户一键「全部移入 SSOT」
4. 移入完成后：SSOT 有 5 个主副本，原位置变 symlink

### 场景 B：跨 Agent 共享
1. 用户在 .claude 装了 `web-fetch`
2. 通过本工具迁入 SSOT
3. 想在 .openeva 也用 → 点 `[O _]` toggle
4. 后端 `ln -s ~/.clawheart/skills/web-fetch ~/.openeva/skills/web-fetch`
5. UI 立刻显示 `[O ✓]`

### 场景 C：批量备份
1. 多选 skill → 「备份选中」
2. zip 结构改为：`<zip>/clawheart-skills/<id>/...`（不再按 Agent 分目录，因为 SSOT 是真源）
3. 旧的「按 Agent 分目录」zip 结构标记为 legacy，仍可恢复

### 场景 D：卸载
1. 详情抽屉点「卸载」
2. 弹窗：「将删除 SSOT 主体 + 移除 .claude / .openeva 共 2 处 symlink，不可恢复」
3. 确认 → 后端先 zip 自动备份 → 再删 → invalidate 列表

## 阶段拆分（实施建议）

### Phase A — 数据模型升级（最关键，前后端 ~6h）
- 后端 `DiscoveredSkill` 加 bindings/in_ssot/content_hash
- discover_all 改写为 SSOT-first 算法
- 前端 hook + Row 显示 bindings 状态（**只读展示**，先不接 toggle）

### Phase B — SSOT 写操作（~4h）
- IPC: move_skill_to_ssot / toggle_skill_binding / uninstall_skill
- Settings 加 SSOT 路径配置 + dry-run 开关
- 前端 toggle 接通 + 确认弹窗

### Phase C — UI 重组（~3h）
- FilterSidebar 三段（概览 / Agent / 鉴定）
- SkillRow 拆为独立组件 + AgentToggleGroup
- DetailDrawer 加 Binding 面板

### Phase D — 收尾（~2h）
- MoveToSsotDialog
- 自动备份钩子
- tsc + cargo + 单测

总工作量约 15h，可分 4 个 PR/会话推进。

## 决策（2026-05-21 用户拍板）

1. **SSOT 路径**：`~/.agents/skills/`（cc-switch 风格，跨工具通用）
2. **dry-run**：**默认关闭**，直接写入
3. **toggle 策略**：**仅 symlink**，平台不支持则提示用户（macOS/Linux 默认支持，Windows 后期处理）
4. **zip 备份结构**：保留按 Agent 分目录（旧结构）
5. **Uninstall**：需要 — 删 SSOT 主体 + 清所有 symlink，UI 必弹二次确认
