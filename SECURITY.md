# Security Policy

## 漏洞披露

**请勿在公开 issue / discussion 上披露安全漏洞。**

通过以下渠道私下报告：

- 邮箱：`security@clawheart.live`（PGP 公钥见 https://clawheart.live/pgp.asc）
- GitHub Security Advisory（仓库 → Security → Advisories → New draft）

我们承诺：
- **72 小时内**首次回复
- **14 天内**给出修复时间表
- 修复发布后协调披露（CVE / GHSA 编号）

## 范围

### In Scope
- `proxy/*`、`security/*`、`storage/*`、`agents/*` 模块的可利用漏洞
- 凭据存储 / Keychain 误用
- Tauri IPC 越权
- CA 证书与 TLS 配置
- 自动更新通道的签名验证
- 公告 feed 的签名验证 / 漂移守护

### Out of Scope
- 已知的 v2.x 待办（见 README §16）
- 仅在 root / sudo 用户下可触发的"漏洞"
- 三方依赖漏洞（请直接上报上游；我们追踪 dependabot 与 cargo-audit）
- 攻击需要物理接触设备的场景

## 安全自治措施

我们对自身代码持续执行：

- **`cargo audit` + `cargo deny`** — 每 PR 跑
- **CodeQL + Trivy** — 每周扫
- **dependabot** — 自动 PR
- **Tauri Capabilities** — 显式 IPC 白名单
- **失败关闭** — 任一安全检查 panic → block
- **CA 私钥** — OS Keychain / DPAPI 加密
- **二进制签名** — macOS 公证 + Windows EV 证书 + Linux minisign
- **审计日志** — 所有拦截事件附 MITRE ATT&CK ID

## SLA

| Severity | First reply | Patch (alpha/beta channel) | GA patch |
|----------|------------|--------------------------|---------|
| Critical | 4h         | 24h                       | 72h     |
| High     | 24h        | 72h                       | 14d     |
| Medium   | 72h        | 14d                       | 30d     |
| Low      | 7d         | 30d                       | 90d     |
