# Phase 1 — Minimal Rust ↔ ggml integration

Status: implemented in source; target-Mac compile/smoke still required

Branch: `rust-rewrite-plan`

## Goal

Prove that a Rust static library can live inside the existing patched llama.cpp
process without changing tensor placement, Metal execution, model output, or KV
behavior.

Phase 1 is intentionally inert. It establishes ABI ownership and build/link
boundaries only.

## Architecture boundary

```text
llama-server / llama-bench / llama-perplexity / validation llama-cli
                |
                v
          llama-common
                |
        common_init() only
                |
                v
       tosh_tier_runtime_create
                |
                v
      libggml_shim.a (Rust)
```

No inference graph callback exists yet. No expert or KV tensor is moved by Rust.

## Existing engine primitives declared for later phases

`ffi/ggml-sys` declares only the patched Metal transfer surface that the tier
runtime is expected to need later:

- `ggml_metal_synchronize`
- `ggml_metal_set_tensor_async`
- `ggml_metal_get_tensor_async`
- `ggml_metal_cpy_tensor_async_ex`

The declarations are not called in Phase 1. This keeps unresolved engine state
out of the Rust policy code until the transfer scheduler is implemented.

## Stable ABI v1

The C boundary is `tosh-tier/include/tosh_tier.h`.

```text
TOSH_TIER_ABI_VERSION = 1
ToshTierConfigV1      = 16 bytes, alignment 4
```

The shared configuration contains only:

- ABI version
- struct size
- reserved flags
- reserved extension field

The Rust runtime rejects ABI and struct-layout mismatches before allocating a
handle.

### Ownership

`ToshTierRuntime *` is opaque to C/C++.

- `tosh_tier_runtime_create` allocates it.
- the caller owns one returned handle.
- `tosh_tier_runtime_destroy` releases it exactly once.
- null destroy is a no-op.
- `common.cpp` keeps the process-global handle and releases it through `atexit`.

All raw-pointer ownership stays inside `ggml-shim`. Pure policy crates remain
free of C pointers.

## Hard fallback

`TOSH_TIER_DISABLE=1` creates the linked runtime in disabled mode.

The engine continues to use its existing static placement exactly as before.
An ABI/create failure also degrades to no Rust runtime rather than aborting the
engine.

`TOSH_TIER_LOG=1` enables only the Phase 1 startup diagnostic, for example:

```text
tosh-tier: linked ABI 1 version 0.1.0-phase1 enabled=no
```

## Build path

The normal `scripts/build-engines.sh` is deliberately unchanged during this
proof phase.

Use:

```bash
zsh scripts/tier-build-engine.sh
```

The wrapper:

1. runs the Rust fmt/clippy/test gate;
2. builds `libggml_shim.a` for the selected macOS architecture;
3. runs the normal ToshLLM llama.cpp build with patches 0001–0010;
4. builds a validation-only `llama-cli` and saves the unmodified binaries under
   `build-static/bin/tier-baseline/`;
5. applies `0011-rust-tier-shim.patch` to that exact patched source tree;
6. points the existing CMake build at the Rust static library and header;
7. relinks `llama-server`, `llama-bench`, `llama-perplexity`, and validation
   `llama-cli`;
8. verifies the Rust ABI symbol exists in each Mach-O binary;
9. starts `llama-server --help` with `TOSH_TIER_DISABLE=1` and requires an inert
   runtime startup diagnostic. `llama-server` calls `common_init()` before
   argument parsing, so this exercises the C++ → Rust lifecycle without a model
   or GPU allocation.

This keeps the experimental seam removable with one patch and avoids making
Rust a prerequisite for the ordinary ToshLLM build before the link experiment
is validated on target hardware.

## cbindgen

The checked-in C ABI has a matching `cbindgen.toml`. To inspect generated output
after an ABI change:

```bash
cargo install cbindgen --locked   # once
zsh scripts/tier-generate-header.sh

diff -u \
  tosh-tier/include/tosh_tier.h \
  tosh-tier/include/tosh_tier.generated.h
```

The generator deliberately writes a separate ignored comparison file by
default. The checked-in header retains the C++ `static_assert`s and is only
replaced after review.

ABI layout is asserted independently in Rust tests and C++ `static_assert`s so
a generator/configuration change cannot silently alter the shared struct.

## Phase 1 validation gates

Required on the target Mac:

```bash
zsh scripts/tier-check-rust.sh
zsh scripts/tier-build-engine.sh
```

Then compare the saved unmodified binaries against the linked-but-disabled
binaries:

```bash
MODEL_PATH=/absolute/path/model.gguf \
N_CPU_MOE=<known-good-static-value> \
zsh scripts/tier-phase1-ab.sh
```

`tier-phase1-ab.sh` runs the same prompt, seed, temperature, model and static
MoE split through both `llama-cli` binaries and byte-compares their generated
stdout. It also captures three-repetition `pp512` / `tg128` benchmark output for
the performance gate. Local A/B artifacts are written beneath
`docs/tier-runtime/phase1-captures/` and ignored by git.

### Correctness gate

- byte-identical deterministic model output for the same model/prompt/seed;
- linked run confirms `TOSH_TIER_DISABLE=1` in its startup diagnostic;
- no crash during startup or shutdown;
- no ABI warnings;
- no runtime handle leak visible in repeated process launches.

### Performance gate

The linked but inert runtime should be within measurement noise of the normal
engine. Target: <= 1% difference in pp/tg medians.

### Build gate

All must pass:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- x86_64 staticlib build on the 2019 Mac Pro
- llama-server link
- llama-bench link
- llama-perplexity link
- validation llama-cli link
- Mach-O symbol check for the Rust ABI
- startup smoke with `TOSH_TIER_DISABLE=1`
- deterministic A/B output equality

## Explicitly not implemented

Phase 1 does not:

- intercept MoE router output;
- call Metal transfer functions from Rust;
- move an expert;
- allocate a GPU cache;
- modify `--n-cpu-moe`;
- tier KV;
- register a custom `ggml_backend_buffer_type`;
- replace any ToshLLM Metal kernel.

Those boundaries are intentional. Phase 2 begins only after the inert ABI/link
path passes on the target Mac.
