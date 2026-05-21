# 接入模式（Access Mode）UI 设计

> 关联：[`borrow-from-pipelock.md` §3](./borrow-from-pipelock.md)
> 目标：让非安全专业小白能在 2 层界面深度内**看懂、选好、改对**自己的接入档位。
> 状态：W4 基座绿灯前置可实现（无需 hudsucker / fetch_server 真实激活）。

---

## 1. 设计原则

1. **首屏即决策**：Onboarding 必经一步选档；不让小白默认在不知情时跑在最弱档位
2. **L1 卡片常驻**：随时可改，工具矩阵里一眼能看见当前档
3. **横向 3 列对比**：一屏看完所有档位差异
4. **ASCII 流程图嵌入卡片**：用图代替术语
5. **切档 = 一次模态引导**：弹窗里走步骤，不跳页（保持 ≤2 层深度）
6. **可回退**：任何时候能降档，无残留

## 2. 信息架构

```
L0 Onboarding（仅首次启动）
   Step 1: 欢迎 + Agent 发现（已有）
   Step 2: 选档位（新增）─────────► 默认推荐"试试看"档

L1 工具矩阵（首页）
   ┌──────────────────────────────┐
   │ 接入模式卡片                  │
   │ 徽章：当前档位（试试看/审计/强制）│
   └──────────────────────────────┘
                │
                ▼
L2 接入模式工具页 (/tools/access-mode)
   横向 3 列：
   ┌──────┬──────┬──────┐
   │试试看 │认真审计│ 强制 │
   │      │ 当前  │      │
   │ASCII │ ASCII │ ASCII│
   │ 图   │  图   │  图  │
   │ ✓ 优 │ ✓ 优  │ ✓ 优 │
   │ ✗ 缺 │ ✗ 缺  │ ✗ 缺 │
   │[切换]│ 当前  │[切换]│
   └──────┴──────┴──────┘

   ┌──────────────────────────────┐
   │ 高级状态（折叠）              │
   │ - CA 证书：已安装 / 未安装    │
   │ - 系统代理：开 / 关           │
   │ - 端口：19111/19112           │
   └──────────────────────────────┘

(L3 模态弹窗，不算独立层级)
   - 切档确认 Dialog
   - CA 安装引导 Dialog（含步骤）
   - Sandbox 命令生成 Dialog（含复制按钮）
```

## 3. IPC 契约

```rust
// src-tauri/src/commands/access_mode.rs
pub enum AccessTier { Tier1, Tier2, Tier3 }   // tier1 = 试试看 / tier2 = 审计 / tier3 = 强制

pub struct AccessModeInfo {
    pub current_tier: String,        // "tier1" | "tier2" | "tier3"
    pub reverse_proxy_port: u16,     // 19112（W12+ fetch_server）
    pub forward_proxy_port: u16,     // 19111（W5 hudsucker）
    pub ca_installed: bool,
    pub ca_path: String,
    pub system_proxy_active: bool,
    pub fetch_url_template: String,  // 给档位 1 用户拷贝
    pub backend_ready: bool,         // 真实代理是否已激活（W5 前总是 false）
}

pub struct CaInstallResult {
    pub ok: bool,
    pub platform: String,            // "macos" | "windows" | "linux"
    pub message: String,
    pub manual_steps: Vec<String>,   // 自动失败时显示
}

pub struct SandboxCommandPreview {
    pub command: String,             // "clawheart sandbox -- python agent.py"
    pub platform: String,
    pub feature_available: bool,     // false 时 UI 显示"v2.x 上线"
    pub notes: Vec<String>,
}

// 命令清单
get_access_mode() -> AccessModeInfo
set_access_mode(tier: String) -> AccessModeInfo
install_ca() -> CaInstallResult
uninstall_ca() -> bool
check_ca_status() -> { installed: bool, fingerprint: Option<String> }
generate_sandbox_command(cmd: String, args: Vec<String>) -> SandboxCommandPreview
```

## 4. 文件清单

### 新增

| 文件 | 职责 |
|------|------|
| `src-tauri/src/commands/access_mode.rs` | 6 个 IPC 命令（W5 前为 stub，状态从 SQLite settings 读写） |
| `src/components/access-mode/AccessModeTool.tsx` | L2 主页 |
| `src/components/access-mode/AccessTierCard.tsx` | 单档卡片 |
| `src/components/access-mode/AccessTierAsciiFlow.tsx` | ASCII 图组件 |
| `src/components/access-mode/SwitchTierDialog.tsx` | 切档确认 |
| `src/components/access-mode/InstallCaDialog.tsx` | CA 安装引导 |
| `src/components/access-mode/SandboxCommandDialog.tsx` | Sandbox 命令生成 |
| `src/components/access-mode/data.ts` | 三档元信息（名称、图、优缺点） |
| `src/hooks/useAccessMode.ts` | TanStack Query 包装 |

### 改动

| 文件 | 改动 |
|------|------|
| `src-tauri/src/lib.rs` | 注册 6 个新命令 |
| `src-tauri/src/commands/mod.rs` | `pub mod access_mode;` |
| `src/App.tsx` | 加 `/tools/access-mode` 路由 + Onboarding 走两步逻辑 |
| `src/components/tools/Onboarding.tsx` | 改成两步式 |
| `src/components/grid/tools.config.ts` | 加 access_mode 卡片 |
| `src/components/grid/HomeHero.tsx` | 工具底栏可能要 +1 项 |
| `src/lib/ipc.ts` | 加 AccessMode 类型 |
| `src/locales/zh.json` + `en.json` | 补 access_mode.* keys |

## 5. 渐进式实施

```
本次 PR（W4 基座绿灯前置）：
✅ IPC + UI 全部到位
✅ 数据持久化（settings 表加 access_mode 列）
✅ 切档操作走 IPC，但真实代理切换为 stub

W5 hudsucker spike 时：
  - set_access_mode("tier2") → 真实启动 forward_proxy
  - install_ca() → 真实调 security add-trusted-cert / certutil

W12 fetch_server 上线时：
  - set_access_mode("tier1") → 真实启动 fetch_server :19112
  - URL 模板返回真实可用地址

W20 sandbox 上线时：
  - generate_sandbox_command() 输出真实可执行命令
  - 在 macOS 与 Linux 上跑通
```

## 6. 反向兼容与降档

- **持久化**：当前档位写入 SQLite `settings.access_mode`
- **重启自恢复**：启动时读 `access_mode` 并尝试激活对应监听器；失败则降档到 tier1 + 弹通知
- **降档无残留**：tier2→tier1 自动停 forward_proxy，但**不自动卸载 CA**（让用户确认）
- **失败关闭语义**：tier2 启动失败时**不应自动降到 tier1**，而是显示"代理未运行 → 流量未保护"，让用户主动决策
