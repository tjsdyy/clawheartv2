# Changelog

All notable changes to ClawHeart Desktop v2 will be documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) · Semver.

## [Unreleased]

### Added (W1 启动周)
- Tauri 2 + Rust + React 项目骨架
- 工具箱范式 UI：12 卡片矩阵 + 10 完整 L2 工具页
- 5 配色主题（Paper 默认 / Carbon / Glacier / Terminal / Cyber）
- ⌘K 命令面板 + 应用内托盘弹窗 + 引导页
- i18n 基础设施（i18next + zh/en + 10 语言占位）
- Rust 安全引擎核心算法：
  - 6 遍归一化（zero-width / 同形字 / Leetspeak / base64 / NFKC / whitespace）
  - 30 条危险指令规则
  - 50 条提示词注入模式
  - 48 类凭据 DLP + Luhn/Mod97/ABA 校验位
  - 4 源 OR Kill Switch
  - 信号分类（Threat/Protective/ConfigMismatch/InfraError）
  - SHA-256 自实现（无外部 crate）
- MCP 深度安全：JSON-RPC 拦截 + 10 攻击链子序列 + 工具基线冻结
- 协议层 L0：5 协议自动检测 + normalizer trait + 完整 5 实现
- 熔断器 4 状态机 + 跨请求字节预算
- Agent 发现：6 平台扫描器（claude/codex/cursor/gemini/windsurf/openclaw）
- 文件漂移监控（sha256 baseline）
- 数据层：完整 v2 schema（17 表）+ v1→v2 迁移程序
- Keychain 凭据存储抽象层
- 40+ IPC commands 完整接口
- CI workflows（fmt+clippy+test 3 平台矩阵 + cargo-audit）
- Release workflow（macOS 公证 + Windows EV + Linux minisign + 自动更新签名）
- 文档：CONTRIBUTING / SECURITY / 迁移指南 / UI 文档 / 架构文档

### Feature flags
- `storage` — rusqlite + keyring
- `proxy_real` — hudsucker + rustls + rcgen
- `agents_real` — sysinfo + notify
- `sync_real` — reqwest
- `feed_verify` — ed25519-dalek
- `updater` — tauri-plugin-updater
- `full` — 全部启用
