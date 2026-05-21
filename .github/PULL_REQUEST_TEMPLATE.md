## What & Why

<!-- 一段话讲清这个 PR 改了什么 + 为什么 -->

Closes #

## Type
- [ ] feat — new feature
- [ ] fix — bug fix
- [ ] docs — docs only
- [ ] refactor — no behavior change
- [ ] perf — performance
- [ ] test — tests only
- [ ] chore — tooling / deps
- [ ] **security** — security relevant (see SECURITY.md)

## Touched modules
<!-- 勾选所有影响的模块；security/proxy/storage 修改需要 2+ reviewers -->
- [ ] `src-tauri/src/security/*`
- [ ] `src-tauri/src/proxy/*`
- [ ] `src-tauri/src/storage/*`
- [ ] `src-tauri/src/agents/*`
- [ ] `src-tauri/src/sync/*`
- [ ] `src-tauri/src/commands/*`
- [ ] `src-tauri/capabilities/*` (IPC whitelist)
- [ ] `src/components/tools/*`
- [ ] `src/components/grid/tools.config.ts` (matrix 配置)
- [ ] `src/styles/globals.css` (themes)
- [ ] `src/locales/*` (i18n)
- [ ] `.github/workflows/*`
- [ ] Docs / README

## Verification

- [ ] `cargo fmt -- --check` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo test --all` passes
- [ ] `pnpm lint` passes (TypeScript)
- [ ] `pnpm build` passes
- [ ] New code has tests
- [ ] If security relevant: regression sample added to `tests/security/`

## Screenshots / Demo (frontend only)

<!-- 如有 UI 改动，附截图 / 短录屏 -->

## Notes for reviewers

<!-- 特别想让 reviewer 注意的地方 / 已知 trade-off / 后续 follow-up -->
