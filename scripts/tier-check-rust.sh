#!/bin/zsh
set -euo pipefail

cd "$(dirname "$0")/.."
cd tosh-tier

echo "== ToshLLM tier runtime: cargo fmt =="
cargo fmt --all -- --check

echo "== ToshLLM tier runtime: cargo clippy =="
cargo clippy --workspace --all-targets -- -D warnings

echo "== ToshLLM tier runtime: cargo test =="
cargo test --workspace

echo "Rust tier scaffold OK"
