#![allow(unsafe_code)]

//! Minimal raw bindings for the patched ggml Metal transfer surface.
//!
//! These declarations mirror the symbols already present after ToshLLM patches
//! `0001-metal-amd-staging-transfers.patch` and
//! `0009-metal-multigpu-dispatch.patch`. Phase 1 does not call them yet; the
//! declarations establish the narrow FFI boundary that later transfer-policy
//! code will use.

use std::ffi::c_void;

#[repr(C)]
pub struct GgmlMetalOpaque {
    _private: [u8; 0],
}

#[repr(C)]
pub struct GgmlTensorOpaque {
    _private: [u8; 0],
}

pub type GgmlMetal = *mut GgmlMetalOpaque;

extern "C" {
    pub fn ggml_metal_synchronize(ctx: GgmlMetal);

    pub fn ggml_metal_set_tensor_async(
        ctx: GgmlMetal,
        tensor: *mut GgmlTensorOpaque,
        data: *const c_void,
        offset: usize,
        size: usize,
    );

    pub fn ggml_metal_get_tensor_async(
        ctx: GgmlMetal,
        tensor: *const GgmlTensorOpaque,
        data: *mut c_void,
        offset: usize,
        size: usize,
    );

    pub fn ggml_metal_cpy_tensor_async_ex(
        ctx_src: GgmlMetal,
        ctx_dst: GgmlMetal,
        src: *const GgmlTensorOpaque,
        dst: *mut GgmlTensorOpaque,
        prefer_events: bool,
    ) -> bool;
}
