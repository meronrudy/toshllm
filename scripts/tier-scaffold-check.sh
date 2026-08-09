#!/bin/zsh
set -euo pipefail

cd "$(dirname "$0")/.."
ROOT="$PWD"

required=(
  tosh-tier/crates/tier-core/src/lib.rs
  tosh-tier/crates/expert-cache/src/lib.rs
  tosh-tier/crates/kv-tier/src/lib.rs
  tosh-tier/crates/transfer-sched/src/lib.rs
  tosh-tier/crates/trace/src/lib.rs
  tosh-tier/crates/host-tier/src/lib.rs
  tosh-tier/crates/policy-sim/src/main.rs
  tosh-tier/crates/ggml-shim/src/lib.rs
  tosh-tier/ffi/ggml-sys/src/lib.rs
  tosh-tier/include/tosh_tier.h
  docs/tier-runtime/scaffold-phases-2-14.md
  docs/tier-runtime/maintainer-test-matrix.md
)

for path in "${required[@]}"; do
  if [ ! -f "$ROOT/$path" ]; then
    echo "missing scaffold path: $path" >&2
    exit 1
  fi
done

zsh "$ROOT/scripts/tier-check-rust.sh"

(
  cd "$ROOT/tosh-tier"
  cargo run -p policy-sim -- 4:1 9:2 4:3 7:4 4:5 9:6
)

echo "Phases 2-14 source scaffold is internally present."
echo "Hardware-dependent hooks and performance claims remain intentionally unvalidated."
