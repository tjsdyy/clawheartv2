# v1 → v2 迁移指南

> 给 v1 已有用户的过渡说明。开发者视角的迁移程序在 `src-tauri/src/storage/migrations.rs`。

## 数据迁移程序

首次启动 v2 时自动检测 `~/.clawheart/local-client.db`（v1 数据库），如未迁移过则：

1. **备份**：原 DB 拷贝到 `~/.clawheart/backups/clawheart-v1-{timestamp}.db.bak`
2. **新建 v2 库**：在独立路径 `~/.clawheart-v2/clawheart.db`（避免污染 v1）
3. **拆密钥到 Keychain**：v1 SQLite 明文里的 api_key / token → OS Keychain
4. **合并 3 张技能表**：disabled / deprecated / user → `skills` 单表
5. **重塑用量表**：`llm_usage_cost_events` → `request_logs` + `token_usage`
6. **写迁移版本号**：`schema_migrations(version=2)`
7. **报告**：迁移 N 条 / 跳过 M 条 / 警告 K 条

## 不兼容变更（提前广播）

| 变更 | 影响 | 应对 |
|------|------|------|
| CORS `*` 取消 | 第三方脚本若直接 fetch `127.0.0.1:19111`，会失败 | release notes 说明：v2 起 19111 只对 LLM SDK 暴露 LLM 兼容路径，UI 由 Tauri IPC 接管 |
| 设置文件格式 | 敏感字段移到 Keychain | 提供 `clawheart export-backup` 命令做完整备份 |
| 完整版 / Core 版统一 | 不再两个安装包 | 首次启动可选择是否安装 OpenClaw |

## 行为等价保证

v1 用户的 SDK 配置（base URL = `http://127.0.0.1:19111/v1`）**无需修改**：

- `/v1/chat/completions` 路径完全保留
- `x-oc-skills` header 治理语义保留
- `apiBase / ocApiKey / llmKey` 三件套语义保留

## 灰度策略

- **v2.0-alpha**：内部团队 + 主动征集；并行 v1（v2 用独立目录）
- **v2.0-beta**：放开下载，"Beta" 角标，允许 v1/v2 来回
- **v2.0-rc**：迁移工具默认开启，引导覆盖 `~/.clawheart`
- **v2.0 GA**：v1 下线下载，已装版本继续可用但停止维护

## FAQ

**Q: 我的 v1 配置/历史会丢吗？**
A: 不会。迁移程序保留完整 v1 备份；任何时候可恢复。

**Q: 必须装 CA 证书才能用 v2 吗？**
A: 是的，HTTPS MITM 需要。引导流程会一步步带你走。
也保留"不装 CA = 降级到正向代理（仅 OpenClaw 路径生效）"作为退路。

**Q: 升级前要不要停 Agent？**
A: 不用。但建议在更新过程中暂时不要让 Agent 跑高负载任务。
