# 架构文档

完整蓝图见仓库外侧：

`../../../opencarapace-server/local-desktop/CLAWHEART_V2_BLUEPRINT_CN.md`

精炼总览见根 `readme.md` §3-§8。

## 8 层防御速查

```
L0  格式归一化     → proxy/format_detector.rs + proxy/normalizers/*.rs
L1  网络层         → proxy/server.rs + proxy/tls.rs (W5+)
L2  内容层         → security/{danger,injection,redact}.rs + normalizer.rs
L3  协议层 (MCP)   → security/{mcp,mcp_chains,mcp_baseline}.rs
L4  供应链层       → security/{skill_scanner,advisory}.rs
L5  系统层         → agents/{scanner,drift}.rs + platforms/*
L6  数据层         → storage/* + security/redact.rs
L7  应急层         → security/kill_switch.rs (4 源 OR)
```

## 关键 trait

| Trait | 在哪里 | 作用 |
|-------|--------|------|
| `SecurityCheck` | `security/mod.rs` | 统一安全检查接口 |
| `RequestNormalizer` | `proxy/normalizer.rs` | 协议格式适配器 |
| `PlatformScanner` | `agents/platforms/mod.rs` | Agent 平台发现 |
| `AuditCheck` | `security/scanner.rs` | 80 项离线审计 |

## 失败关闭

全局原则：**任一安全检查 panic / timeout → 走 block 路径**。
具体在 `proxy/pipeline.rs::process` 实现。
