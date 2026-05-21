# Proxy 测试

## R1 spike — hudsucker 24h 真实流量压测

W5 第一周必须跑通：

```bash
# 1. 启动 hudsucker hello world
cd tests/proxy
cargo run --bin hudsucker-spike

# 2. 配置 Claude Code / Codex 走 127.0.0.1:19111
export ANTHROPIC_BASE_URL=http://127.0.0.1:19111
export OPENAI_BASE_URL=http://127.0.0.1:19111

# 3. 让真实 SDK 跑 24h（脚本化高频请求）
./scripts/24h-soak.sh
```

### 绿灯指标
- 24h 无 panic / 无 mem leak（RSS < 40MB）
- 1k QPS × 30 min 稳定
- SSE 流式响应**逐字**透传（端到端延迟 ≤ 上游 + 5ms）

### 失败回退
如果 hudsucker 不达标 → 切自研 `hyper + rustls` 中间层（参考 mitmproxy_rs 架构）。
开发计划文档 §10 R1 已写明 4 周缓冲。
