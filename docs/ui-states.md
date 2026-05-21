# UI State Reference

## 工具卡片徽章状态

| Badge Kind | 视觉 | 数据来源 | 含义 |
|-----------|-----|---------|------|
| `alert`     | 红色 pill + 白色脉动点 | 后端 `intercept_events.severity=critical` 今日数 | 关键拦截 |
| `alert-high` | 橙色 pill + 白色脉动点 | severity=high 今日数 | 高风险 |
| `count`     | 灰色数字 | 业务计数 | 上次时间 / 总数 |
| `soon`      | 灰色 SOON 角标 | tools.config.ts `status=coming_soon` | 即将上线 |
| 无          | — | — | 无新事件 |

## 主题切换 surface

5 主题对应 CSS class（应用在 `<html>`）：

| Theme | Class | Default |
|-------|-------|---------|
| Paper · 羊皮纸 | `theme-paper` | ✅ |
| Carbon · 深空墨 | `theme-carbon` | |
| Glacier · 冰川蓝 | `theme-glacier` | |
| Terminal · 终端绿 | `theme-terminal` | |
| Cyber · 赛博紫 | `theme-cyber` | |

切换 → localStorage `clawheart-theme` 持久化 + IPC `set_theme` 同步到后端 settings 表（W3 起）。

## 严重等级配色

| Severity | Token | Hex (Paper) | Hex (Carbon) |
|----------|-------|-------------|--------------|
| critical | `--critical` | `#DC2626` | `#EF4444` |
| high     | `--high`     | `#EA580C` | `#F97316` |
| medium   | `--medium`   | `#D97706` | `#FBBF24` |
| low      | `--low`      | `--accent` | `--accent` |

## 工具色（11 个工具独立 accent）

| Tool | Hue (Paper) | Tool | Hue (Paper) |
|------|------------|------|-------------|
| monitor | blue-600 #2563EB | budget | green-600 #16A34A |
| scan | orange-600 #EA580C | audit | zinc-500 #71717A |
| skills | purple-600 #9333EA | token_verify | amber-500 #EAB308 |
| advisory | red-600 #DC2626 | openclaw | indigo-600 #4F46E5 |
| logs | cyan-600 #0891B2 | relay | pink-500 #EC4899 |
| | | policy | slate-500 #64748B |

## 浮层互斥

`useOverlays` zustand store 保证三个 overlay 互斥：
- `cmdkOpen` (⌘K 命令面板)
- `trayOpen` (托盘弹窗)
- `themePickerOpen` (主题切换器)

任一打开 → 其他自动关闭。

## 状态条数据来源

```
● 防护中 · 19111 · CA ✓ · ↑320KB ↓2.1MB · 同步 14:21    v2.0.0-alpha.0
```

| 字段 | 来源 |
|------|------|
| 防护中/已暂停 | `status.kill_switch ? "已暂停" : "防护中"` |
| `19111` | `status.proxy_port` |
| `CA ✓` | `status.ca_trusted` |
| 流量 | W6 接 proxy worker，alpha mock |
| 同步 | W18 接 sync worker，alpha mock |
| 版本 | `status.version` |

## 引导页阶段

```
1. 首启动（onboarding.completed === false）
   ↓ 点"开始发现"
2. 扫描中 (1.2s mock delay)
   ↓
3. setCompleted(true) → 进入工具矩阵
```

跳过演示模式也走第 3 步。重置：`localStorage.removeItem("clawheart-onboarding")`。

## ⌘K 命令面板

```
Pattern: ⌘K (Mac) / Ctrl+K (Win/Linux) → 唤起
方向键 ↑↓ → 选中
Enter → 执行
Esc → 关闭
```

命令分 3 段：**推荐** / **导航** / **工具**。
搜索（`input` 监听 `query` state）过滤匹配项。
