#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[1/4] cargo check --workspace"
cargo check --workspace

echo "[2/4] cargo test --workspace"
cargo test --workspace

echo "[3/4] npm ci"
npm ci

echo "[4/4] npm run build"
npm run build

echo
echo "Preflight complete."
echo "Note: 'cargo run' requires GPU access and will fail in headless environments."
