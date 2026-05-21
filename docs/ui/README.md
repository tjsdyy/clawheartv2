# UI 文档

## 范式：工具箱 + 2 层深度

详细方案见仓库外：
`../../../opencarapace-server/local-desktop/.context/ui-mocks-v2.md`

高保真 HTML 原型：
`../../../opencarapace-server/local-desktop/.context/ui-prototype-v2.html`

## 代码落点

| UI 概念 | 代码 |
|---------|------|
| L1 工具矩阵 | `src/components/grid/{ToolsGrid,ToolCard,tools.config}.tsx` |
| L2 通用壳 | `src/components/tools/ToolLayout.tsx` |
| 主壳（header + status） | `src/components/AppShell.tsx` |
| ⌘K 命令面板 | `src/components/overlays/CommandPalette.tsx` |
| 托盘弹窗 | `src/components/overlays/TrayPopup.tsx` |
| 主题切换器 | `src/components/overlays/ThemePicker.tsx` |
| 5 主题 CSS | `src/styles/globals.css` |
| 11 工具色 token | `src/styles/globals.css` (`--tool-*`) |

## 强制约束（lint 规则）

- 禁止 `tools/*` 子组件 navigate 到 `/tools/*` 之外的路径
- 禁止 L2 子组件嵌 `<Routes>` / `<Router>`
- 矩阵 `tools[]` 配置变更必须过 PR review
- 禁止内联 style 属性（替换为 Tailwind / CSS variables）—— W14 起 lint 强制
