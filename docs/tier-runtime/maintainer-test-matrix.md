# Maintainer / AMD Hardware Test Matrix

This branch was scaffolded without access to a 2019 Mac Pro. The useful next step is empirical validation by a ToshLLM maintainer or community tester with Intel Mac + AMD Metal hardware.

## Minimum useful hardware

Any Intel Mac with a supported AMD GPU can validate build/link and read-only tracing. The highest-value configurations are:

- 2019 Mac Pro, 24/28-core Xeon, large RAM configuration
- Vega II / Vega II Duo
- W5700X / W6800X / W6900X
- multiple AMD GPUs, especially an Infinity Fabric pair
- a large-memory host capable of running the 35B and 122B MoE cases already used in ToshLLM testing

## Test A — Rust workspace

```bash
git clone https://github.com/meronrudy/toshllm.git
cd toshllm
git checkout rust-rewrite-plan
zsh scripts/tier-check-rust.sh
```

Please report Rust toolchain, architecture, and any fmt/clippy/test failure verbatim.

## Test B — inert engine link

```bash
zsh scripts/tier-build-engine.sh
```

Expected:

- baseline binaries saved under `vendor/llama.cpp/build-static/bin/tier-baseline/`
- Rust ABI symbol present in linked binaries
- `TOSH_TIER_DISABLE=1` startup succeeds
- no placement or transfer behavior changes

## Test C — deterministic A/B

```bash
MODEL_PATH=/path/to/model.gguf \
N_CPU_MOE=<known-good-value> \
zsh scripts/tier-phase1-ab.sh
```

Please report:

- whether stdout is byte-identical
- baseline pp512/tg128
- Rust-linked-disabled pp512/tg128
- percentage delta

## Test D — synthetic policy scaffold

```bash
cd tosh-tier
cargo run -p policy-sim -- 4:1 9:2 4:3 7:4 4:5 9:6
```

This only proves the offline crate wiring; it is not a performance result.

## Test E — router-hook review

Before any live tracing patch is added, please identify/review the narrowest patched-llama.cpp location where per-token/per-layer top-k MoE expert IDs are already available on the CPU side without forcing a Metal synchronization.

The intended call contract is:

```c
tosh_tier_record_router_selection(
    runtime,
    sequence_id,
    token_position,
    layer,
    expert_ids,
    expert_count);
```

If expert IDs are not naturally host-visible at the right point, the tracing design should change rather than introducing a sync.

## Test F — target trace corpus

Once the read-only hook is accepted, collect at least:

1. normal chat session
2. coding-agent/tool-use session
3. 32k+ long-context session
4. deliberate topic/domain switch
5. concurrent requests if supported

Preferred models:

- current 35B MoE test model
- current 122B MoE test model

Required trace context:

- model/quant fingerprint
- `--n-cpu-moe`
- token position
- layer
- selected experts
- GPU/VRAM snapshot sampled without a synchronization
- existing transfer telemetry where already available

## What would justify live Phase 4 work

Proceed only if real traces show a meaningful working set. Suggested initial gate:

- >=70% simulated expert-cache hit rate at a realistic VRAM budget, or
- >=25% simulated reduction in bytes crossing the host/device boundary.

If neither is present, keep the project as an instrumentation result and do not add a live mover.

## Requested feedback

Please post:

- hardware configuration
- exact command
- commit SHA
- full failure/success log
- benchmark table
- whether the proposed router hook can remain synchronization-free
- any conflict you see with existing persistent staging, prompt cache, disk KV persistence, MTP, or multi-GPU scheduling
