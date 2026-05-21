# Contributing to ClawHeart Desktop v2

谢谢参与！本项目守护用户在 AI 流量上的主权，**自身代码质量必须比同类工具都干净**（参见 `readme.md` §11 安全自治原则）。

## 开发流程

1. **Issue 先行**：实现型 PR 必须 link 到一个 issue；纯文档/typo 例外
2. **分支命名**：`feat/<scope>`、`fix/<scope>`、`docs/<scope>`、`refactor/<scope>`、`chore/<scope>`
3. **小 PR**：一个 PR 一个改动主题；大重构请拆分
4. **测试**：所有新逻辑须带单元测试；安全相关须带回归样本
5. **lint 必过**：`cargo fmt --check`、`cargo clippy -- -D warnings`、`pnpm lint`

## 强制 Code Review 规则

| 模块 | Reviewer 要求 |
|------|--------------|
| `src-tauri/src/proxy/*` | 至少 2 名核心组成员，含 1 名安全审 |
| `src-tauri/src/security/*` | 至少 2 名核心组成员，含 1 名安全审 |
| `src-tauri/src/storage/*` | 至少 1 名核心组成员 |
| `src/components/grid/tools.config.ts` | 必须过架构组（新增工具卡片要审 namespace） |
| 其他 | 至少 1 名 maintainer |

## 安全敏感操作

下列变更必须额外注明并通过安全 review：
- `security/danger.rs` 规则新增/修改
- `security/injection.rs` 模式调整
- `security/redact.rs` 凭据类增删
- `security/kill_switch.rs` 触发源调整
- `security/advisory.rs` 公钥变更（漂移守护必须 3 处副本一致）
- `proxy/normalizer.rs` 新增协议
- `capabilities/default.json` IPC 白名单

## 提交格式

约定式提交（Conventional Commits）：

```
<type>(<scope>): <subject>

<body>

<footer>
```

type 可选：`feat | fix | docs | style | refactor | perf | test | chore | security`

## 报告漏洞

**不要**在公开 issue 里贴漏洞。见 `SECURITY.md`。

## 行为准则

参与即同意遵守 [Contributor Covenant](https://www.contributor-covenant.org/) 2.1。
