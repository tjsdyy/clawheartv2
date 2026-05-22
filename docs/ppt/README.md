# ClawHeart 新手快速上手 PPT

面向新手的产品操作流程介绍，从安装到使用全程，HTML 单文件格式（无依赖、本地直接打开、可分享）。

## 目录结构

```
docs/ppt/
├── index.html           ← 打开它就能看 PPT
├── SCREENSHOTS.md       ← 31 张截图清单 + 文件名约定
├── README.md            ← 你正在看的文件
└── screenshots/         ← 把图片放这里
    ├── 01-install-dmg.png
    ├── 02-drag-to-applications.png
    └── ...
```

## 使用步骤

1. **截图**：照 `SCREENSHOTS.md` 拍 31 张，按指定文件名放到 `screenshots/`
2. **预览**：双击 `index.html` 在浏览器打开（缺图的页面会显示占位）
3. **调整标注**：如果某页的红圈位置不准，在 `index.html` 里搜索对应文件名，调整 `style="top: X%; left: Y%; ..."` 的百分比即可
4. **打印 PDF**：浏览器 `Cmd+P` 即可导出 PDF（已优化 print 样式）

## 操作

| 键位 | 作用 |
|------|------|
| `←` / `→` | 上下页 |
| `Space` / `PageDown` | 下一页 |
| `PageUp` | 上一页 |
| `Home` / `End` | 跳到首页/末页 |
| `T` | 打开目录 |
| 鼠标点击左 1/3 区域 | 上一页 |
| 鼠标点击右 2/3 区域 | 下一页 |

## 标注系统说明

HTML 内置 3 种标注，全部用 CSS 百分比定位（不依赖图片像素绝对值）：

```html
<!-- 红色数字编号圆 -->
<div class="ann ann-num" style="top: 50%; left: 30%;">1</div>

<!-- 红色矩形高亮（带 pulse 光晕） -->
<div class="ann ann-rect" style="top: 30%; left: 20%; width: 60%; height: 12%;"></div>

<!-- 黄色文字气泡 -->
<div class="ann ann-tip" style="top: 75%; left: 50%; transform: translateX(-50%);">说明文字</div>
```

## 当前进度

- 总页数：**33 页**（封面 1 + 介绍 1 + 操作 31 + 总结 1）
- 截图清单：**31 张** 已在 `SCREENSHOTS.md` 列好
- 章节：1.安装 → 2.首次启动 → 3.发现 Agent → 4.模型渠道 → 5.反向导入 → 6.启用托管 → 7.监控与回滚 → 8.设置 → 9.进阶
