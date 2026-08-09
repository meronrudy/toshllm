#![allow(unsafe_code)]
#![allow(clippy::must_use_candidate)]

//! Minimal raw bindings for the patched ggml/Metal transfer surface.
//!
//! The crate intentionally exposes only symbols already provided by ToshLLM's
//! patched engine plus opaque handles reserved for the later buffer-type seam.
//! Policy crates never import raw ggml pointers directly.

use std::ffi::c_void;

#[repr(C)]
pub struct GgmlMetalOpaque {
    _private: [u8; 0],
}

#[repr(C)]
pub struct GgmlTensorOpaque {
    _private: [u8; 0],
}

#[repr(C)]
pub struct GgmlBackendBufferTypeOpaque {
    _private: [u8; 0],
}

pub type GgmlMetal = *mut GgmlMetalOpaque;
pub type GgmlBackendBufferType = *mut GgmlBackendBufferTypeOpaque;

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
