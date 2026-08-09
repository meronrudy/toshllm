#!/bin/zsh
set -euo pipefail

cd "$(dirname "$0")/.."
ROOT="$PWD"
OUT="${TOSH_TIER_BASELINE_DIR:-$ROOT/docs/tier-runtime/baseline-captures/$(date -u +%Y%m%dT%H%M%SZ)}"
mkdir -p "$OUT"

if [ "$(uname -s)" != "Darwin" ]; then
    echo "tier-capture-baseline.sh must run on macOS" >&2
    exit 1
fi

MODEL_PATH="${MODEL_PATH:-}"
SERVER_BINARY="${SERVER_BINARY:-$ROOT/vendor/llama.cpp/build-static/bin/llama-server}"
BENCH_BINARY="${BENCH_BINARY:-$ROOT/vendor/llama.cpp/build-static/bin/llama-bench}"
N_CPU_MOE="${N_CPU_MOE:-0}"
BENCH_PP="${BENCH_PP:-512}"
BENCH_TG="${BENCH_TG:-128}"

BRANCH="$(git rev-parse --abbrev-ref HEAD)"
COMMIT="$(git rev-parse HEAD)"
VERSION="$(tr -d '[:space:]' < VERSION)"
LLAMA_COMMIT="$(sed -n 's/^LLAMA_COMMIT="${LLAMA_COMMIT:-\([^}]*\)}".*/\1/p' scripts/build-engines.sh | head -1)"

cat > "$OUT/run-metadata.txt" <<EOF
captured_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)
branch=$BRANCH
commit=$COMMIT
tosh_version=$VERSION
llama_commit=$LLAMA_COMMIT
model_path=$MODEL_PATH
server_binary=$SERVER_BINARY
bench_binary=$BENCH_BINARY
n_cpu_moe=$N_CPU_MOE
bench_pp=$BENCH_PP
bench_tg=$BENCH_TG
EOF

# system_profiler already emits valid JSON and captures installed RAM, CPU and
# all visible GPUs without adding jq/python dependencies to the project.
system_profiler SPHardwareDataType SPMemoryDataType SPDisplaysDataType -json > "$OUT/hardware.json"

sysctl -a 2>/dev/null | grep -E '^(hw\.(memsize|ncpu|physicalcpu|logicalcpu)|machdep\.cpu\.brand_string)' > "$OUT/sysctl.txt" || true

cat > "$OUT/patch-stack.txt" <<'EOF'
0001-metal-amd-staging-transfers.patch
0005-turboquant-kv.patch
0006-mrope-kv-shift.patch
0007-wave64-reductions-and-quants.patch
0009-metal-multigpu-dispatch.patch
0010-mtp-kv-only-catchup.patch
EOF

if [ -n "$MODEL_PATH" ]; then
    if [ ! -f "$MODEL_PATH" ]; then
        echo "MODEL_PATH does not exist: $MODEL_PATH" >&2
        exit 1
    fi
    if [ ! -x "$BENCH_BINARY" ]; then
        echo "llama-bench not found/executable: $BENCH_BINARY" >&2
        echo "Run ./scripts/build-engines.sh first." >&2
        exit 1
    fi

    bench_args=(-m "$MODEL_PATH" -p "$BENCH_PP" -n "$BENCH_TG")
    if [[ "$N_CPU_MOE" -gt 0 ]]; then
        bench_args+=(--n-cpu-moe "$N_CPU_MOE")
    fi

    echo "Running baseline llama-bench..."
    printf 'args=' >> "$OUT/run-metadata.txt"
    printf '%q ' "${bench_args[@]}" >> "$OUT/run-metadata.txt"
    printf '\n' >> "$OUT/run-metadata.txt"

    # Keep the raw canonical output; Benchmark.swift remains the source of truth
    # for ToshLLM's richer saved result format. This capture is intentionally
    # non-invasive and does not alter the app benchmark path.
    "$BENCH_BINARY" "${bench_args[@]}" 2>&1 | tee "$OUT/llama-bench.txt"
else
    cat > "$OUT/README.txt" <<'EOF'
Hardware and engine metadata captured. No benchmark was run because MODEL_PATH
was not provided.

Example:
  MODEL_PATH=/path/model.gguf N_CPU_MOE=20 zsh scripts/tier-capture-baseline.sh
EOF
fi

echo "Phase 0 baseline capture written to: $OUT"
