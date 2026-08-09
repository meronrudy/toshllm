#!/bin/zsh
set -euo pipefail

cd "$(dirname "$0")/.."
ROOT="$PWD"
OUT="${1:-$ROOT/tosh-tier/include/tosh_tier.h}"

if ! command -v cbindgen >/dev/null 2>&1; then
    echo "cbindgen is not installed." >&2
    echo "Install it with: cargo install cbindgen --locked" >&2
    exit 1
fi

cd "$ROOT/tosh-tier"
cbindgen \
    --config cbindgen.toml \
    --crate ggml-shim \
    --output "$OUT"

echo "generated $OUT"
