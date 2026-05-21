# 技能备份功能 — 实施计划

## 目标

把首页"技能市场"卡片改造为"技能备份"，提供端到端能力：
1. **发现** 本机所有 Agent 安装的技能（按 Agent 归属）
2. **安全鉴定扫描** 每个技能（复用现有 `skill_scanner`）
3. **打包备份** 选中技能 → zip 导出

## Agent 范围（通配发现）

**策略**：扫描用户主目录下所有 `~/.<agent_name>/skills/` 目录，自动捕获任意 Agent。`agent_name` 即点号后的 dotfile 名（如 `claude` / `openeva` / `codex` / `clawcode` / `openclaw` / `clawheart`）。

```rust
// 伪代码
for dir in fs::read_dir(home_dir)? {
    let name = dir.file_name();
    if !name.starts_with('.') { continue; }
    let skills_dir = dir.path().join("skills");
    if !skills_dir.is_dir() { continue; }
    let agent_name = name.trim_start_matches('.').to_string();
    // 枚举 skills_dir 下每个子目录作为一个 skill
    for skill in fs::read_dir(skills_dir)? {
        push(DiscoveredSkill { source_agent: agent_name.clone(), ... });
    }
}
```

**自动支持**：claude / openeva / codex / clawcode / openclaw / clawheart / cursor / continue / 任何未来新工具，零代码改动。

**Skill 识别条件**：子目录中**任一**条件满足即视为 skill：
- 含 `SKILL.md`（首选 — frontmatter + body 格式）
- 含 `manifest.json` / `package.json` 且声明 `name`
- 含 `*.md` + 至少 1 个可执行/脚本文件

**例外**：`.git` / `.vscode` / `.DS_Store` 等系统 dotfile 跳过

## 后端实施

### 1. 依赖（Cargo.toml）

新增：`zip = "2.2"` （主依赖，不放 feature 后面 —— 备份是核心能力）

### 2. 新模块 `src-tauri/src/skills/`

```
skills/
├── mod.rs
├── discover.rs    # 扫描各 Agent 路径
├── manifest.rs    # 解析 SKILL.md frontmatter
└── backup.rs      # zip 打包
```

**DiscoveredSkill 模型**：

```rust
pub struct DiscoveredSkill {
    pub id: String,           // hash(source_agent + path)
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub source_agent: String, // "claude" | "openeva" | "codex" | "clawcode" | ...
    pub source_path: String,  // 绝对路径
    pub file_count: u32,
    pub total_bytes: u64,
    pub manifest_excerpt: Option<String>,
}
```

### 3. IPC（替换/扩展 `commands/skills.rs`）

```rust
#[tauri::command]
pub fn discover_local_skills() -> AppResult<Vec<DiscoveredSkill>>;

#[tauri::command]
pub fn scan_local_skill(id: String) -> AppResult<ScanReport>;
// 调用 skills.rs::backup 写到用户选的目录
#[tauri::command]
pub fn backup_skills(ids: Vec<String>, output_zip_path: String) -> AppResult<BackupResult>;
```

**BackupResult**：

```rust
pub struct BackupResult {
    pub zip_path: String,
    pub skill_count: u32,
    pub total_bytes: u64,
}
```

zip 结构：

```
clawheart-skills-backup-2026-05-19.zip
├── manifest.json   # 整个备份的元数据 + 每个 skill 的归属 + 扫描分数
├── claude/
│   ├── skill-a/...
│   └── skill-b/...
├── openeva/
│   └── skill-c/...
└── codex/
    └── ...
```

## 前端实施

### 4. Hooks（`src/hooks/useSkills.ts` 扩展或新文件）

```ts
useDiscoveredSkills()      // → DiscoveredSkill[]
useScanLocalSkill(id)      // 按需扫描
useBackupSkills()          // mutation: { ids, output_path } → BackupResult
```

### 5. 新页面 `SkillsBackupTool.tsx`（替换 `SkillsTool`）

布局：

```
┌─────────────┬────────────────────────────────────────────┐
│ Agent 列表  │  搜索栏 + 全选 + 扫描全部 + 备份选中        │
│ ─────────── │ ──────────────────────────────────────────  │
│ ☑ Claude    │  ☑ skill-name @0.1.2  · 8 文件 · 24KB      │
│   12 个技能  │     ★ 评分 95 · 安全                       │
│ ☑ OpenEva   │     [扫描] [详情]                          │
│   8 个技能   │  ☑ another-skill ...                       │
│ ☐ Codex     │                                            │
│   0 个技能   │                                            │
└─────────────┴────────────────────────────────────────────┘
```

### 6. tools.config.ts 重命名

```diff
- { id: "skills", label: "技能市场", description: "142 启用 / 184", ...
+ { id: "skills", label: "技能备份", description: "发现 · 鉴定 · 打包", ...
```

### 7. App.tsx 路由

tabs 从 ["全部", "已安装", "推荐", "安全", "最新"] 改为：

- ["本机技能", "扫描报告", "备份历史"]
- 视图 0 = 主列表
- 视图 1 = 已扫描的详细 findings
- 视图 2 = 历史备份 zip 列表（后续 phase B 加 DB）

## 阶段拆分

### Phase A（首版可演示，目标本次完成）
- A1 后端 skills::discover + manifest 解析
- A2 后端 IPC: discover_local_skills + backup_skills
- A3 前端 hooks + SkillsBackupTool 极简页（左侧 Agent + 右侧列表 + 备份按钮）
- A4 tools.config.ts 改名 + App.tsx 移除旧 tabs
- A5 删除旧 SkillsTool.tsx
- A6 自检 tsc + cargo

### Phase B（增量）
- 安全扫描接入 UI（scan 按钮 + findings 详情）
- 备份历史 DB 表 + 视图
- Cursor / Continue 支持
- 单个 skill 详情抽屉（文件树 / SKILL.md 预览）

## 风险 / 待确认

- "各种 claw" 具体范围 — 当前按 ClawCode / OpenClaw / ClawHeart 三个生态名假设；如有其他需补充
- Codex CLI 的 skills 目录约定 — 当前按 `~/.codex/instructions/` 假设；用户验证后调整
- 备份不应包含敏感文件（.env / .git/credentials）— 在 backup.rs 加忽略名单
