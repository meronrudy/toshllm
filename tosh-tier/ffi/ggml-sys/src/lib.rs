#![allow(unsafe_code)]

//! Raw bindings boundary for the patched ggml/llama.cpp engine.
//!
//! Phase 0 intentionally exposes no engine symbols. Phase 1 will add only the
//! minimum bindings required by the tier runtime.
