# Icons

Tauri 2 `bundle.icon` 期望以下尺寸（生产构建时必需）：

- `32x32.png`
- `128x128.png`
- `128x128@2x.png`
- `icon.icns` (macOS)
- `icon.ico` (Windows)

**Dev 模式（`pnpm tauri:dev`）不需要这些文件**——可以直接跑。

## 首次 bundle 前

准备一张 ≥1024×1024 的 PNG 源图（推荐 `clawheart.png`），然后跑：

```bash
pnpm tauri icon ./path/to/clawheart.png
```

会自动生成所有平台需要的尺寸到本目录。

## 临时占位

如果想立刻 bundle 测试，可以用 `../../public/clawheart.svg` 转一张 PNG：

```bash
# macOS（使用系统自带 sips）
qlmanage -t -s 1024 -o . ../../public/clawheart.svg
mv clawheart.svg.png clawheart.png
pnpm tauri icon ./clawheart.png
```
