# TODO

## 方案 A：以 Agent 为主重构（已选定 2026-05-21）

### 阶段 1 · 命名修复（已完成 2026-05-21）
- [x] ProvidersTool 文案：Profile → 模型渠道、新建中转 Profile → 新建模型渠道
- [x] BatchHistoryView 文案：历史批次 → 应用记录
- [x] OverwriteWizard 文案：选 Profile → 选渠道、模型管理 → 历史批次 → 应用记录
- [x] ImportFromAgentsDialog 文案：N 个 Profile → N 个模型渠道、手动建 Profile → 手动建渠道
- [x] en.json: Provider Profiles → Model Channels

### 阶段 2 · AgentDetail 页结构（已完成 2026-05-21）
- [x] 新建 `src/components/agents/AgentDetailPage.tsx`
- [x] Header（agent 元数据 + 状态 chip）+ 截获方式 inline 备注（带「更改」跳转）
- [x] 模型渠道列表（profile cards · 复用 useProviderProfiles · 单条「应用到此 Agent」/「编辑」）
- [x] 候选 Agent 引导块（显示 discovery_signals）
- [x] 底部操作（请求日志 / 安全检查 / 复制路径）
- [x] 路由：`/tools/agents/detail?platform=xxx&name=yyy`
- [x] AgentsTool 卡片可点击进入详情（candidate 不可点击）

### 阶段 3 · 增强（已完成 2026-05-21）
- [x] 「应用到此 Agent」直接弹 OverwriteWizard 并锁定 agent（跳过 Step 2）
- [x] Agent 卡片 + 详情页显示「当前生效渠道」（扫最近 10 个 batch 推断）
- [x] Agent 列表卡片显示 "当前渠道：xxx" 或 "未配置模型渠道"

### 阶段 4 · 监控模式 + 模型管理收敛
- [ ] 监控模式页加 "查看每个 Agent 当前所用 Tier" 列表
- [ ] 模型管理仍可独立维护（仓库视角保留），但首页底栏入口降级
- [ ] 全局 Tier chip 在顶栏不动

### 阶段 5 · 候选决策持久化（已完成 2026-05-21）
- [x] 加 IPC：confirm_unknown_agent / ignore_unknown_agent / reset / list
- [x] settings 表存 `agent.unknown_decisions = JSON{platform: confirmed|ignored}`
- [x] commands::agents::list_agents 拉取 storage 过滤 ignored、升级 confirmed → active

### 阶段 6 · 监控模式 Agent 列表（已完成 2026-05-21）
- [x] AccessModeTool 底部加 `AgentTierOverrideList`
- [x] 显示所有已纳入 Agent + 当前所用 Tier（v2.0 共用全局，v2.1 加单独覆盖）
- [x] 点击 Agent 进入详情页

### 阶段 7 · Agent 详情页应用记录（已完成 2026-05-21）
- [x] `AgentApplyHistory` 子组件
- [x] 扫最近 10 个 batch 的 snapshots，过滤 agent_id 匹配的
- [x] 按 applied_at desc 显示：渠道名 + 时间 + 已回滚标签
- [x] 单条回滚按钮（dry-run 模式提示）

### 阶段 8 · cc-switch 风格收敛（已完成 2026-05-21）
- [x] AgentsTool 重写为 tab + 渠道列表（cc-switch 风格）
- [x] 顶部 Agent tab 切换、右上角刷新 + 加号
- [x] 渠道行：状态点 + 名称 + base_url + 当前使用徽章 + 一键「启用」
- [x] 候选 Agent 折叠到底部，含 confirm/ignore
- [x] 删除 `AgentDetailPage.tsx` + 路由 `/tools/agents/detail`
- [x] 首页底栏移除「模型管理」卡片（路由保留作为新建/编辑入口）
- [x] AgentTierOverrideList 点击改为返回 Agent 列表

### 阶段 9 · 预设模型 + 新增渠道弹层（已完成 2026-05-21）
- [x] `data/provider-presets.ts` — 60+ 主流供应商预设（cc-switch + 9router 合并）
- [x] 按 category 分组（官方/国内大厂/云服务/聚合路由/第三方/自定义）
- [x] 每个预设品牌色 + 首字母色块图标
- [x] `AddChannelDialog.tsx` — cc-switch 风格 Preset 网格 + 表单
- [x] 选中预设自动填 base_url + name + notes
- [x] 推荐排序：根据当前 Agent tab 平台优先排序匹配预设
- [x] 调 useCreateProvider 一次性创建 profile + 设凭据
- [x] AgentsTool `+` 按钮改为打开 Dialog（不再跳 /tools/providers）
- [x] 监控模式 Tier1 卡片「模型管理」按钮 → 「Agent 管理」，跳 /tools/agents

### 阶段 11 · Agent 托管控制面板（已完成 2026-05-21）
- [x] 调研 9router 改写策略：merge 保留用户其他字段 + soft reset（删除标记字段）
- [x] ClawHeart 沿用 apply_overwrite 现有路径：snapshots 表存 before_value 真实备份，比 9router soft reset 可靠
- [x] 新建 `AgentTakeoverPanel.tsx` —— 托管状态 banner（绿色已托管 / 灰色未托管）
- [x] 显示：状态徽章 + 生效渠道 + 配置文件路径 + Dry-run 标记
- [x] 操作：[历史 N] 弹抽屉 / [关闭托管] 调 rollback_snapshot
- [x] 新建 `AgentHistoryDrawer.tsx` —— Agent 维度的应用记录抽屉
- [x] 按 applied_at desc 列出本 agent 所有 snapshot，单条回滚
- [x] AgentsTool 集成：tab 下方插入 banner
- [x] merge 策略提示：「ClawHeart 采用 merge 策略改写配置文件，原值持久化备份」

### 阶段 10 · 编辑/OAuth/品牌图标增强（已完成 2026-05-21）
- [x] **品牌 SVG 图标**：新建 `BrandIcon.tsx`，6 个主流厂商 simple-icons path（Anthropic / OpenAI / Codex / Google Gemini / GitHub / Nvidia / AWS），其他 fallback 字母色块
- [x] **编辑模式 Dialog**：AddChannelDialog 加 `editingProfile` prop，编辑模式隐藏预设网格、API Key 留空保留、调 useUpdateProvider + 可选 setCredential
- [x] **OAuth 占位**：preset 加 `auth_method`，OAuth 类（GitHub Copilot / OpenAI Codex）显示 v2.1 提示 + chip 上加 KeyRound 标记
- [x] **recommended_for 自动推断**：根据 protocol + category 自动补全（聚合类同时推荐 claude+codex）
- [x] AgentsTool 渠道行用 BrandIcon + hover 齿轮按钮打开编辑 Dialog
- [x] 「配置凭据」按钮也走编辑 Dialog（不再跳 /tools/providers）

### 后续优化
- [ ] PATH 可执行文件多路径扫描（nvm/fnm/volta）
- [ ] `{tool} --version` 提取 semver
- [ ] Claude Desktop 平台 scanner（macOS + Windows）
