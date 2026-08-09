#![allow(unsafe_code)]

/// Phase 0 boundary only. No ggml symbols are called until Phase 1.
pub const TOSH_TIER_ABI_VERSION: u32 = 1;

#[no_mangle]
pub extern "C" fn tosh_tier_abi_version() -> u32 {
    TOSH_TIER_ABI_VERSION
}
