# Rust Weight/KV Tiering Runtime — Phases 2–14 Scaffold

Status: source scaffold only; no target AMD/Mac Pro hardware validation has been claimed.

Branch: `rust-rewrite-plan`

## Boundary

The scaffold keeps the existing ToshLLM/llama.cpp Metal backend authoritative for execution. Rust owns proposed policy and telemetry surfaces only. No Rust code in this branch replaces AMD Flash Attention, ToshGEMM, wave64 kernels, persistent staging, peer/event transfers, or all-reduce.

## Phase 2 — read-only tracing

Implemented interfaces:

- `trace::RouterSelection`
- ranked `ExpertChoice` records
- VRAM samples
- transfer samples
- bounded `VecTrace` with dropped-event accounting
- C ABI counters: `tosh_tier_record_router_selection`, `tosh_tier_record_vram_sample`, `tosh_tier_runtime_stats`

Still required on the patched engine: choose the narrowest router dispatch point where top-k expert IDs are already materialized and call the read-only ABI. The hook must not add a GPU synchronization.

## Phase 3 — offline policy simulation

Implemented interfaces:

- `ExpertCacheState`
- LRU and LFU ranking
- expert activation statistics
- `TransitionTable` for simple next-expert prediction
- `policy-sim` command-line scaffold

Example synthetic run:

```bash
cd tosh-tier
cargo run -p policy-sim -- 4:1 9:2 4:3 7:4
```

Real trace replay remains pending until Phase 2 emits target-hardware traces.

## Phase 4–6 — live expert cache and predictive prefetch

Implemented policy/state types:

- cache budgets and per-GPU ownership
- minimum-residency and re-evaluation intervals
- promotion/eviction action vocabulary
- demand/prefetch/background transfer priorities
- bounded transfer queue where demand always wins
- transition-probability predictor scaffold

Not implemented: any live expert pointer substitution or Metal submission. Those are intentionally blocked on hardware traces and maintainer review.

## Phase 7 — multi-GPU placement

Implemented:

- `GpuId`
- multi-tier residency model
- `TransportKind::{HostStage, EventHandoff, PeerCopy}`
- measured-link cost abstraction

The future runtime should feed measured topology costs into placement while continuing to call the existing patched `ggml_metal_cpy_tensor_async_ex(..., prefer_events)` path.

## Phase 8 — host weight tier

New `host-tier` crate scaffolds:

- immutable mapped-region specifications
- host-memory budgets
- memory-pressure states
- VM advice policy (`Sequential`, `Random`, `WillNeed`, `DontNeed`)

Actual `mmap`/`madvise` calls are not implemented because they require a small audited unsafe/macOS boundary and target-machine measurement.

## Phase 9 — KV tier

`kv-tier` now models:

- per-sequence/per-layer KV blocks
- token ranges and byte size
- generation/version identity
- recent-window policy
- spill-to-host and restore-to-VRAM decisions

No existing ToshLLM prompt-cache or disk-backed conversation cache is modified.

## Phase 10 — generic ggml buffer type

Not registered yet. `tier-core` now has the placement vocabulary needed for a future `ggml_backend_buffer_type` adapter. Registration is deliberately deferred until the MoE-specific path proves useful.

## Phase 11 — correctness/reliability

Current pure-Rust unit tests cover core residency invariants, cache ranking, KV classification, bounded traces, and transfer priority. Phase 1 already supplies deterministic linked-vs-unlinked A/B scripts. Target hardware must add long-run soak, output equality/perplexity checks, and memory-growth measurements.

## Phase 12 — adaptive policy

`PolicyMode` includes `Static`, `Lru`, `Lfu`, `Windowed`, `Predictive`, and `Adaptive`. No self-tuning loop is enabled; this is an API reservation only.

## Phase 13 — ToshLLM UI/benchmark integration

No SwiftUI changes are included. The existing benchmark UI remains authoritative. Once hardware data exists, the runtime can expose optional settings and the existing benchmark records can gain tier-policy metadata.

## Phase 14 — experimental release

Release is blocked on all of the following:

1. Rust workspace fmt/clippy/test passes.
2. x86_64 static library links into the pinned patched engine.
3. Phase 1 deterministic A/B passes.
4. Router trace hook is reviewed and does not synchronize the GPU.
5. Real 35B/122B MoE traces show useful locality.
6. Offline simulation predicts meaningful transfer savings.
7. Live cache wins against static `--n-cpu-moe` without correctness regressions.
8. Multi-GPU topology is measured rather than assumed.
9. Long-context KV spill does not conflict with existing cache paths.
10. Eight-hour soak shows bounded RAM/VRAM/resource usage.

## Current scaffold map

```text
tosh-tier/
├── crates/
│   ├── tier-core/       placement vocabulary + budgets + residency
│   ├── expert-cache/    activation stats, ranking, transition predictor
│   ├── kv-tier/         KV block/window policy
│   ├── transfer-sched/  bounded priority queues + topology cost
│   ├── trace/           bounded read-only trace events
│   ├── host-tier/       host mapping/memory-pressure policy
│   ├── policy-sim/      offline simulator CLI scaffold
│   └── ggml-shim/       C ABI/lifecycle + telemetry counters
└── ffi/ggml-sys/        raw patched Metal declarations
```

The remaining work is hardware validation and narrow engine-hook integration, not another architecture rewrite.
