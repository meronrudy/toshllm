# Phase 0 — Rust Weight/KV Tiering Runtime

Status: implemented scaffold; hardware benchmark capture must be executed on the target Mac Pro.

Branch: `rust-rewrite-plan`

## Boundary

Phase 0 does not alter llama.cpp scheduling, Metal kernels, tensor placement, KV placement, or ToshLLM's Swift runtime. It establishes a Rust workspace, an explicit FFI safety boundary, and reproducible baseline-capture tooling.

The existing compute layer remains authoritative:

- llama.cpp pinned by `scripts/build-engines.sh` to `571d0d540` unless explicitly overridden.
- `0001-metal-amd-staging-transfers.patch`
- `0005-turboquant-kv.patch`
- `0006-mrope-kv-shift.patch`
- `0007-wave64-reductions-and-quants.patch`
- `0009-metal-multigpu-dispatch.patch`
- `0010-mtp-kv-only-catchup.patch`

## Rust workspace

`tosh-tier/` contains the planned crate boundaries:

- `tier-core`: tier and identity types. Pure Rust; unsafe forbidden.
- `expert-cache`: MoE policy domain. Pure Rust; unsafe forbidden.
- `kv-tier`: KV tier domain. Pure Rust; unsafe forbidden.
- `transfer-sched`: transfer intent/policy domain. Pure Rust; unsafe forbidden.
- `trace`: tracing schema/domain. Pure Rust; unsafe forbidden.
- `ggml-shim`: future C ABI boundary. Unsafe permitted only here when Phase 1 requires it.
- `ffi/ggml-sys`: future raw patched-ggml bindings. Unsafe permitted only here.

No ggml symbols are bound or called in Phase 0.

## Validate the Rust scaffold

```bash
zsh scripts/tier-check-rust.sh
```

This runs:

```text
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Capture the target hardware baseline

Metadata-only capture:

```bash
zsh scripts/tier-capture-baseline.sh
```

Benchmark capture after the current engines are built:

```bash
MODEL_PATH=/absolute/path/to/model.gguf \
N_CPU_MOE=20 \
zsh scripts/tier-capture-baseline.sh
```

Each capture is written beneath `docs/tier-runtime/baseline-captures/<UTC timestamp>/` and includes:

- `hardware.json`: raw macOS `system_profiler` JSON for CPU/RAM/DIMMs/GPUs.
- `sysctl.txt`: key CPU and memory values.
- `run-metadata.txt`: ToshLLM branch/commit/version, llama.cpp pin, model and benchmark settings.
- `patch-stack.txt`: patch-order snapshot.
- `llama-bench.txt`: raw baseline output when `MODEL_PATH` is supplied.

Do not commit model paths or captures containing information you do not want public.

## Phase 0 completion criteria

Code/scaffold criteria:

- [x] working branch exists.
- [x] Rust workspace exists.
- [x] crate boundaries match the tier-runtime architecture.
- [x] pure-Rust crates forbid unsafe code.
- [x] FFI is isolated to `ggml-shim` / `ggml-sys`.
- [x] Rust toolchain, formatting and lint policy are pinned.
- [x] reproducible hardware/baseline capture tooling exists.
- [x] current llama.cpp pin and patch stack are documented.

Target-hardware criteria, intentionally not fabricated in-repo:

- [ ] run hardware capture on the 2019 Mac Pro.
- [ ] record 35B-class MoE baseline.
- [ ] record 122B-class MoE baseline when available.
- [ ] record one dense control-model baseline.
- [ ] capture `pp512`, `tg128`, model load behavior, VRAM/RAM state and current static `--n-cpu-moe` configuration.

Phase 1 must not begin changing the engine hot path until the Rust scaffold passes and at least the primary 35B baseline is captured on the actual target machine.
