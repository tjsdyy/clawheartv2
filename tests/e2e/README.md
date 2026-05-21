# E2E 测试

W22 起接入 Playwright + Tauri WebDriver。覆盖：

## 必跑场景

1. **首次启动 onboarding 流程**
   - 跳过 → 矩阵可用
   - 完成发现 → 矩阵 + Agent 数显示

2. **核心导航**
   - L1 → L2 (监控/扫描/技能/Agent/设置)
   - 返回按钮回 L1
   - 跨工具跳转保持 2 层深度

3. **5 配色切换**
   - 主题切换 → CSS variables 应用
   - localStorage 持久化 → 重启保持

4. **⌘K 命令面板**
   - 快捷键唤起
   - 方向键 + Enter 触发
   - esc 关闭

5. **Kill Switch 流程**
   - 点击 → 状态条变红
   - IPC 命令 trigger → backend state 更新

6. **多语言切换** (W14 起)
   - zh / en 切换无重启
   - 关键文案全覆盖

## 运行

```bash
pnpm exec playwright test
```
