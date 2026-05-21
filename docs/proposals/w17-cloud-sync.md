# W17：云端同步（策略下发 + 拦截日志聚合）

> 关联：[`borrow-from-pipelock.md`](./borrow-from-pipelock.md) §5（规则包生态）、`sync/` 已有骨架
> 状态：设计阶段（pending review）· 拟 W17 实施
> 前置依赖：本机端 v2 GA + 云端 API（独立仓库 clawheart-cloud）

---

## 0. 定位：什么云端同步，什么不同步

ClawHeart 是 **local-first** 安全运行时——核心检测能力必须在断网时仍可用。云端的角色：
- ✅ **同步规则更新**（危险指令、注入模式、攻击链）
- ✅ **聚合企业级审计**（团队管理员看全员拦截）
- ✅ **公告与漂移情报**（CVE 类公告 + 签名漂移 → 主动通知用户）
- ❌ **不存储用户的 LLM 流量原文**（隐私第一）
- ❌ **不强制在线**（断网时本机功能 100% 可用）

---

## 1. 5 类实体同步契约

每个实体 = 一个 SyncEntity，独立 worker、独立游标、独立 fail-safe：

| 实体 | 方向 | 频率 | 失败行为 |
|------|------|------|---------|
| `danger_rules` | 云 → 本机 | 6h / 手动 | 本机保留上次 |
| `injection_patterns` | 云 → 本机 | 6h | 本机保留上次 |
| `mcp_baselines`（社区） | 云 → 本机 | 24h | 本机保留上次 |
| `security_advisories` | 云 → 本机 | 1h（公告型）| 本机保留上次 |
| `intercept_summary` | 本机 → 云（企业版） | 实时（批量）| 本机缓冲队列，最大 7 天 |

每实体走相同 trait：

```rust
// src-tauri/src/sync/entity.rs (已有骨架)
pub trait SyncEntity: Send + Sync {
    fn name(&self) -> &'static str;
    fn period(&self) -> Duration;

    /// 拉取增量
    async fn pull(&self, cursor: Option<String>) -> Result<SyncBatch, SyncError>;

    /// 推送本机增量（仅 outgoing 类）
    async fn push(&self, items: Vec<SerializedItem>) -> Result<(), SyncError>;

    /// 写入本机存储
    fn apply_local(&self, batch: SyncBatch) -> Result<()>;
}
```

---

## 2. 鉴权模型

```
本机端                                 云端
─────────────────────                ─────────────────────
1. 用户在 ClawHeart 登录              POST /auth/google
   Google OAuth                       验证 ID token
                                      签发 JWT (1h)
                                  ◀── + Refresh Token (30d, Keychain)
                                      
2. 每次 sync 请求                     GET /api/sync/danger?cursor=...
   Bearer <jwt>                  ──▶  鉴权 → 返回增量
                                  ◀── { items, next_cursor, max_age }

3. JWT 过期                          POST /auth/refresh
                                  ◀── 新 JWT
```

**关键决策**：
- JWT 短期 + Refresh Token 长期 → 减少 Refresh API 调用频次
- Refresh Token 存 Keychain（与 LLM 凭据同等级保护）
- 鉴权失败一律重新走 Google OAuth；不缓存密码
- 企业版另加 SSO（SAML/OIDC）

---

## 3. 增量同步协议

### 3.1 服务端响应格式

```jsonc
GET /api/sync/danger_rules?cursor=2026-05-19T08:00:00Z

{
  "items": [
    {
      "op": "upsert",
      "rule_id": "DG-031",
      "pattern": "...",
      "version": 42,
      "signature": "ed25519:..."
    },
    {
      "op": "delete",
      "rule_id": "DG-007",
      "version": 8
    }
  ],
  "next_cursor": "2026-05-19T14:00:00Z",
  "max_age_seconds": 21600,
  "feed_signature": "ed25519:整批的签名"
}
```

### 3.2 本机应用前的硬性校验

```rust
// 1. 整批签名校验（防中间人篡改 + 兜底）
verify_ed25519(feed_signature, items.serialize(), CLOUD_PUBLIC_KEY)?;

// 2. 单条签名校验
for item in items {
    verify_ed25519(item.signature, item.serialize_canonical(), CLOUD_PUBLIC_KEY)?;
}

// 3. 版本递增检查（防回滚攻击）
if item.version <= local_version(item.id) {
    skip();
}

// 4. 写入数据库
db.upsert_or_delete(item);
```

公钥硬编码在 Rust 源码（CI 时 gitleaks 检查防泄露）；签名密钥仅在云端 KMS 内。

### 3.3 离线兜底

```
启动时：检查每个实体的 last_sync_at
  ├─ < max_age_seconds → 用本机缓存，跳过 pull
  ├─ ≥ max_age_seconds 但 < 7 天 → 本机缓存 + UI 提示"规则数据陈旧 X 小时"
  └─ ≥ 7 天 → UI 显著告警"安全规则严重陈旧，建议联网更新"
```

---

## 4. intercept_summary 上传（企业版）

### 4.1 严格脱敏

**绝不上传**：
- prompt 原文 / response 原文
- 完整 URL（含 query string）
- API key（含部分掩码）
- 用户名 / 邮箱（除非企业版明确开启）

**仅上传**：
```jsonc
{
  "event_id": "uuid",
  "timestamp": "...",
  "agent_platform": "cursor",   // 类型，非用户名
  "rule_hit": "DG-005",          // 规则 ID，非内容
  "severity": "high",
  "mitre_attack_id": "T1552.004",
  "redacted_snippet_hash": "sha256:...",  // 哈希，非原文
  "verdict": "block",
  "layer": "L2.DLP"
}
```

### 4.2 上传节流与重试

```rust
pub struct InterceptUploader {
    queue: VecDeque<RedactedEvent>,
    last_flush: Instant,
}

impl InterceptUploader {
    pub async fn flush(&mut self) -> Result<(), SyncError> {
        // 批量上传：每 30s 或 100 事件触发一次
        if self.queue.len() < 100 && self.last_flush.elapsed() < Duration::from_secs(30) {
            return Ok(());
        }

        let batch: Vec<_> = self.queue.drain(..min(100, self.queue.len())).collect();
        match self.api.upload_intercepts(&batch).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // 失败：放回队列头，但限制重试 3 次
                self.queue.extend(batch);
                if self.consecutive_failures > 3 {
                    // 7 天后队列爆 → 丢弃最旧的（fail-open，不阻塞本机）
                }
                Err(e)
            }
        }
    }
}
```

---

## 5. 已有骨架与接通点

```
src-tauri/src/sync/                      （已有，待激活）
├── mod.rs
├── worker.rs                            ← W17 实现 SyncWorker 调度循环
└── entities/
    ├── advisories.rs                    ← pull GET /api/sync/advisories
    ├── danger.rs                        ← pull GET /api/sync/danger_rules
    ├── skills.rs                        ← pull GET /api/sync/skills
    ├── baselines.rs                     ← pull GET /api/sync/mcp_baselines
    └── policies.rs                      ← v2.2 企业版策略下发
```

W17 实施需要：
1. 启用 `sync_real` feature（reqwest 已引入）
2. 加 `dep:ed25519-dalek`（已存在为 `feed_verify` feature 的依赖）
3. 在 `state.rs` 加 `pub sync_status: Arc<RwLock<SyncStatus>>`
4. 在 lib.rs setup 时 spawn SyncWorker（每个实体一个 tokio task）

---

## 6. 云端 API 契约

`clawheart-cloud` 是独立项目（v2.x 由 v1 opencarapace-server 迁移），关键端点：

```
认证：
  POST /auth/google                  Google ID token → JWT + Refresh
  POST /auth/refresh                 Refresh token → 新 JWT
  POST /auth/logout                  注销
  GET  /auth/me                      当前用户信息

同步（拉取）：
  GET  /api/sync/danger_rules        cursor + max_age + Ed25519 签名
  GET  /api/sync/injection_patterns  同上
  GET  /api/sync/mcp_baselines       同上
  GET  /api/sync/advisories          公告 + 是否匹配本机 fingerprint

同步（推送，企业版）：
  POST /api/sync/intercepts          批量上传脱敏事件
  POST /api/sync/usage_summary       日级 token 用量汇总（可选）

仪表板（管理员）：
  GET  /api/admin/intercepts         查看团队全员拦截
  GET  /api/admin/agents             团队所有 Agent 清单
  POST /api/admin/policy             下发策略（企业版 v2.2）

公告管理：
  GET  /api/advisories               全量公告（不需鉴权）
  POST /api/advisories               发布公告（管理员）
```

每个端点强制 HTTPS + JWT；公告 GET 例外（提供给未登录用户）。

---

## 7. UI 视图

```
设置 → 云端同步                    （W17 新增 section）
─────────────────────────────────────────────────
  账号：已连接 user@example.com    [退出登录]
  
  实体             上次同步         最新版本   状态
  ───────────────  ──────────────  ─────────  ─────
  危险指令         2 分钟前         v142       ✓ 同步
  注入模式         2 分钟前         v88        ✓ 同步
  MCP 基线         32 分钟前        v9         ✓ 同步
  安全公告         5 分钟前         v23        ✓ 同步
  拦截事件上传     1 分钟前         52 / 100   ⚠ 队列已满  
                                              [立即上传]
  
  [立即拉取所有] [禁用同步] [清除本地缓存]
```

---

## 8. 实施路线

```
W17.1（3 天）：基础设施
  - sync_real feature 启用
  - sync/worker.rs 实现 SyncWorker（per-entity tokio task）
  - 鉴权流程（Google OAuth + JWT + Refresh）

W17.2（3 天）：4 个 pull 实体
  - danger_rules / injection_patterns / mcp_baselines / advisories
  - Ed25519 签名校验
  - 增量游标管理

W17.3（2 天）：intercept_summary 上传
  - 严格脱敏
  - 节流批量
  - 失败重试与缓冲

W17.4（2 天）：UI
  - 设置 → 云端同步 section
  - 实时状态显示
  - 手动触发与控制

W17.5（4 天）：云端 API 实现（独立仓库）
  - clawheart-cloud Spring Boot 项目
  - 5 个 entity 的增量分页
  - JWT 鉴权与 RBAC
  - Ed25519 签名（KMS）
```

---

## 9. 隐私与合规

| 关注点 | 处理 |
|--------|------|
| GDPR / 数据出境 | 默认不上传任何用户原文；intercept_summary 字段 + 哈希 |
| 同意机制 | 首次开启上传需明确二次确认（参考 W8 实际写入开关） |
| 数据保留 | 云端默认 90 天滚动；用户可一键清除 |
| 注销 → 数据删除 | 注销时清除云端所有用户数据（GDPR Right to be Forgotten） |
| 企业版数据归属 | 团队管理员持有，员工只读 |
| 加密 | 传输 HTTPS + Ed25519 签名；存储 PostgreSQL RDS 加密 |

---

## 10. 风险与缓解

| 风险 | 缓解 |
|------|------|
| 云端宕机影响本机使用 | local-first：本机仍用上次的规则；UI 仅显示"规则陈旧 X 小时" |
| 签名密钥泄露 | 公钥更换流程：发布新二进制 + 版本协商 + 强制升级 |
| 用户 JWT 被盗用 | 强制 IP 绑定（可选）+ 异地登录通知 + 一键撤销所有会话 |
| 拦截事件上传被中间人篡改 | 客户端先用 Ed25519 私钥签名（per-device key 存 Keychain），云端验签 |
| 队列爆掉丢事件 | 7 天滚动 + 关键事件（severity=critical）优先级抢占 |
| 用户不想用云 | 完全可关：禁用同步后所有实体走本机内置规则；UI 隐藏云端 section |

---

## 11. 不在 W17 范围

- ❌ P2P 同步 / 分布式（不必要复杂）
- ❌ 端到端加密 prompt 内容（不上传，无需加密）
- ❌ 实时推送（WebSocket） → 5h 拉取已足够；v2.x 再说
- ❌ 多设备策略同步 → v2.2 企业版做

---

## 12. 衡量成功

- 本机 sync 完整循环 < 10 秒（4 实体 × 平均 2.5s）
- 离线 7 天后规则数据无变化（除 UI 告警外）
- 拦截事件上传无重复无丢失（基于幂等 event_id）
- 用户首次开启同步到看到云端规则生效 < 60 秒
