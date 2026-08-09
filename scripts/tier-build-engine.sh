#!/bin/zsh
set -euo pipefail

cd "$(dirname "$0")/.."
ROOT="$PWD"
ARCH="${ARCH:-$(uname -m)}"

case "$ARCH" in
    x86_64) RUST_TARGET="x86_64-apple-darwin" ;;
    arm64)  RUST_TARGET="aarch64-apple-darwin" ;;
    *)
        echo "Unsupported ARCH for tier runtime: $ARCH" >&2
        exit 1
        ;;
esac

if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo is required for the experimental Rust tier runtime" >&2
    exit 1
fi

if command -v rustup >/dev/null 2>&1 && ! rustup target list --installed | grep -qx "$RUST_TARGET"; then
    echo "Rust target $RUST_TARGET is not installed." >&2
    echo "Install it with: rustup target add $RUST_TARGET" >&2
    exit 1
fi

echo "== Phase 1: validate Rust workspace =="
"$ROOT/scripts/tier-check-rust.sh"

echo "== Phase 1: build Rust static library for $RUST_TARGET =="
(
    cd "$ROOT/tosh-tier"
    MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-14.0}" \
        cargo build --release --target "$RUST_TARGET" -p ggml-shim
)

TIER_LIB="$ROOT/tosh-tier/target/$RUST_TARGET/release/libggml_shim.a"
TIER_INCLUDE="$ROOT/tosh-tier/include"

if [ ! -f "$TIER_LIB" ]; then
    echo "Rust static library not found: $TIER_LIB" >&2
    exit 1
fi

echo "== Phase 1: build the normal ToshLLM llama.cpp baseline =="
# Keep the production build script unchanged during the Phase 1 experiment.
# It establishes the exact patched source/cache first; the tier seam is then
# applied as one additional patch and the same CMake tree is relinked.
SKIP_IMAGE=1 ARCH="$ARCH" "$ROOT/scripts/build-engines.sh"

cd "$ROOT/vendor/llama.cpp"
# llama-cli is validation-only; the normal ToshLLM build does not need to ship it.
cmake --build build-static --config Release -j "$(sysctl -n hw.ncpu)" -t llama-cli

BIN="$ROOT/vendor/llama.cpp/build-static/bin"
BASELINE_BIN="$BIN/tier-baseline"
rm -rf "$BASELINE_BIN"
mkdir -p "$BASELINE_BIN"
for tool in llama-server llama-bench llama-perplexity llama-cli; do
    cp "$BIN/$tool" "$BASELINE_BIN/$tool"
done
[ ! -f "$BIN/default.metallib" ] || cp "$BIN/default.metallib" "$BASELINE_BIN/default.metallib"

echo "saved unmodified A/B binaries to $BASELINE_BIN"

echo "== Phase 1: apply optional Rust tier seam =="
TIER_PATCH="$ROOT/patches/0011-rust-tier-shim.patch"
git apply --check "$TIER_PATCH"
git apply "$TIER_PATCH"

cmake -B build-static \
    -DTOSH_TIER_LIB:FILEPATH="$TIER_LIB" \
    -DTOSH_TIER_INCLUDE_DIR:PATH="$TIER_INCLUDE"
cmake --build build-static --config Release -j "$(sysctl -n hw.ncpu)" \
    -t llama-server llama-bench llama-perplexity llama-cli

echo "== Phase 1: prove the Rust ABI is present in engine binaries =="
for tool in llama-server llama-bench llama-perplexity llama-cli; do
    path="$BIN/$tool"
    if [ ! -x "$path" ]; then
        echo "missing engine binary: $path" >&2
        exit 1
    fi
    if ! nm -gU "$path" | grep -q '_tosh_tier_abi_version$'; then
        echo "$tool does not contain the Rust tier ABI symbol" >&2
        exit 1
    fi
    echo "linked: $tool"
done

echo "== Phase 1: startup smoke test with hard kill switch =="
# llama-server calls common_init() before argument parsing, so --help is enough
# to exercise the C++ -> Rust lifecycle without loading a model or touching a GPU.
SMOKE_OUT="$(TOSH_TIER_LOG=1 TOSH_TIER_DISABLE=1 "$BIN/llama-server" --help 2>&1 || true)"
printf '%s\n' "$SMOKE_OUT"
if ! printf '%s\n' "$SMOKE_OUT" | grep -q 'tosh-tier: linked ABI 1 .* enabled=no'; then
    echo "Rust tier runtime did not report an inert startup through common_init" >&2
    exit 1
fi

echo "Phase 1 link/lifecycle smoke test passed."
echo "The runtime is linked but performs no placement, transfer, or KV work."
echo "Run scripts/tier-phase1-ab.sh with MODEL_PATH set for deterministic A/B validation."
