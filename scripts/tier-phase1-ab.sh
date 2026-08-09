#!/bin/zsh
set -euo pipefail

cd "$(dirname "$0")/.."
ROOT="$PWD"
BIN="$ROOT/vendor/llama.cpp/build-static/bin"
BASELINE_BIN="$BIN/tier-baseline"
OUT="${TOSH_TIER_AB_DIR:-$ROOT/docs/tier-runtime/phase1-captures/$(date -u +%Y%m%dT%H%M%SZ)}"

MODEL_PATH="${MODEL_PATH:-}"
N_CPU_MOE="${N_CPU_MOE:-0}"
N_PREDICT="${N_PREDICT:-64}"
PROMPT="${PROMPT:-Explain why a deterministic cache policy needs a stable baseline in exactly three concise paragraphs.}"
RUN_BENCH="${RUN_BENCH:-1}"

if [ -z "$MODEL_PATH" ]; then
    echo "MODEL_PATH is required" >&2
    echo "Example: MODEL_PATH=/path/model.gguf N_CPU_MOE=20 zsh scripts/tier-phase1-ab.sh" >&2
    exit 1
fi
if [ ! -f "$MODEL_PATH" ]; then
    echo "MODEL_PATH does not exist: $MODEL_PATH" >&2
    exit 1
fi

for tool in llama-cli llama-bench; do
    if [ ! -x "$BASELINE_BIN/$tool" ]; then
        echo "missing Phase 1 baseline binary: $BASELINE_BIN/$tool" >&2
        echo "Run zsh scripts/tier-build-engine.sh first." >&2
        exit 1
    fi
    if [ ! -x "$BIN/$tool" ]; then
        echo "missing Rust-linked binary: $BIN/$tool" >&2
        exit 1
    fi
done

mkdir -p "$OUT"

COMMON_ARGS=(
    -m "$MODEL_PATH"
    -p "$PROMPT"
    -n "$N_PREDICT"
    --temp 0
    --seed 42
    --no-display-prompt
    --single-turn
    --no-conversation
    --no-show-timings
    --color off
    --log-disable
)
if [ "$N_CPU_MOE" -gt 0 ]; then
    COMMON_ARGS+=(--n-cpu-moe "$N_CPU_MOE")
fi

cat > "$OUT/config.txt" <<EOF
captured_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)
model_path=$MODEL_PATH
n_cpu_moe=$N_CPU_MOE
n_predict=$N_PREDICT
prompt=$PROMPT
commit=$(git rev-parse HEAD)
EOF

echo "== Phase 1 A/B: deterministic token-output proxy =="
"$BASELINE_BIN/llama-cli" "${COMMON_ARGS[@]}" \
    > "$OUT/baseline.stdout" 2> "$OUT/baseline.stderr"

TOSH_TIER_DISABLE=1 TOSH_TIER_LOG=1 \
    "$BIN/llama-cli" "${COMMON_ARGS[@]}" \
    > "$OUT/tier-disabled.stdout" 2> "$OUT/tier-disabled.stderr"

if [ ! -s "$OUT/baseline.stdout" ]; then
    echo "baseline llama-cli produced empty stdout; inspect $OUT/baseline.stderr" >&2
    exit 1
fi

if ! cmp -s "$OUT/baseline.stdout" "$OUT/tier-disabled.stdout"; then
    echo "FAIL: Rust-linked disabled engine changed deterministic model output" >&2
    diff -u "$OUT/baseline.stdout" "$OUT/tier-disabled.stdout" || true
    exit 1
fi

DIGEST="$(shasum -a 256 "$OUT/baseline.stdout" | awk '{print $1}')"
echo "PASS: output byte-identical"
echo "sha256=$DIGEST"

if ! grep -q 'tosh-tier: linked ABI 1 .* enabled=no' "$OUT/tier-disabled.stderr"; then
    echo "FAIL: linked run did not confirm disabled Rust runtime" >&2
    exit 1
fi

if [ "$RUN_BENCH" = "1" ]; then
    echo "== Phase 1 A/B: performance capture (3 repetitions) =="
    BENCH_ARGS=(-m "$MODEL_PATH" -p 512 -n 128 -r 3)
    if [ "$N_CPU_MOE" -gt 0 ]; then
        BENCH_ARGS+=(--n-cpu-moe "$N_CPU_MOE")
    fi

    "$BASELINE_BIN/llama-bench" "${BENCH_ARGS[@]}" \
        > "$OUT/baseline-bench.txt" 2> "$OUT/baseline-bench.stderr"
    TOSH_TIER_DISABLE=1 "$BIN/llama-bench" "${BENCH_ARGS[@]}" \
        > "$OUT/tier-disabled-bench.txt" 2> "$OUT/tier-disabled-bench.stderr"

    echo "benchmark outputs captured for pp512/tg128 median comparison"
fi

echo "Phase 1 A/B artifacts: $OUT"
