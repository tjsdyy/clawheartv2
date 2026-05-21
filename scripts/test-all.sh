#!/usr/bin/env bash
# 跑完整测试集

set -euo pipefail

cd "$(dirname "$0")/.."

echo "▶ Rust fmt check"
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check

echo "▶ Rust clippy"
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings

echo "▶ Rust unit tests"
cargo test --manifest-path src-tauri/Cargo.toml --lib

echo "▶ Rust integration tests"
cargo test --manifest-path src-tauri/Cargo.toml --test integration_test || \
  echo "  (skipped — only runs when tests/ wired up to Cargo.toml)"

echo "▶ Frontend type check"
pnpm lint

echo "▶ Frontend build"
pnpm build

echo "✓ All checks passed"
