#!/usr/bin/env node
// 生成 GitHub Social Preview 图（1280×640 PNG）
//
// 流程：
//   1. 跑起 clawheartv2Web 的 production build（或假设它已在 :3000）
//   2. headless 浏览器打开首页
//   3. 截 hero 区域裁剪到 1280×640
//   4. 输出到 docs/social-preview.png
//
// 用法：
//   pnpm dlx playwright install chromium  # 首次
//   node scripts/capture-social-preview.mjs
//
// 然后到 GitHub: Settings → General → Social preview → Upload 该 PNG。

import { spawn } from "node:child_process";
import { mkdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..");
const WEB_DIR = path.resolve(REPO_ROOT, "..", "clawheartv2Web");
const OUTPUT = path.resolve(REPO_ROOT, "docs", "social-preview.png");

const URL_BASE = process.env.SOCIAL_PREVIEW_URL || "http://localhost:3000";
const LOCALE = process.env.SOCIAL_PREVIEW_LOCALE || ""; // "" = en (default), "zh" = 中文
const WAIT_MS = Number(process.env.SOCIAL_PREVIEW_WAIT_MS ?? 2500);

async function main() {
  const { chromium } = await import("playwright").catch(() => {
    console.error("[err] playwright 未安装。先跑：");
    console.error("      cd " + WEB_DIR + " && pnpm add -D playwright");
    console.error("      pnpm dlx playwright install chromium");
    process.exit(1);
  });

  await mkdir(path.dirname(OUTPUT), { recursive: true });

  // 检测是否能连到本地 web；如果不行，引导用户先 pnpm dev
  const reachable = await fetch(URL_BASE)
    .then((r) => r.ok || r.status < 500)
    .catch(() => false);
  if (!reachable) {
    console.error(`[err] 无法连接 ${URL_BASE}。先在 ${WEB_DIR} 跑：`);
    console.error("      pnpm dev   # 或 pnpm build && pnpm start");
    process.exit(1);
  }

  const target = LOCALE ? `${URL_BASE}/${LOCALE}` : URL_BASE;
  console.log(`[..] launching chromium → ${target}`);

  const browser = await chromium.launch();
  try {
    // GitHub Social Preview 标准：1280×640 (2:1)
    // 用 deviceScaleFactor=2 拿到 retina 质量
    const context = await browser.newContext({
      viewport: { width: 1280, height: 640 },
      deviceScaleFactor: 2,
    });
    const page = await context.newPage();
    await page.goto(target, { waitUntil: "networkidle" });
    // 等动画稳定一帧
    await page.waitForTimeout(WAIT_MS);

    // 截 viewport（已经是 1280×640），不裁剪
    const buf = await page.screenshot({
      type: "png",
      fullPage: false,
      omitBackground: false,
    });
    const { writeFile } = await import("node:fs/promises");
    await writeFile(OUTPUT, buf);
    const kb = (buf.length / 1024).toFixed(1);
    console.log(`[ok] ${OUTPUT}  (${kb} KB, 2560×1280 @2x)`);
    console.log("");
    console.log("  → GitHub: Settings → General → Social preview → Upload");
    console.log("    https://github.com/tjsdyy/clawheartv2/settings");
  } finally {
    await browser.close();
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
