#![allow(unsafe_code)]

use std::os::raw::c_char;
use std::ptr;

/// Stable C ABI version for the experimental ToshLLM tier runtime.
pub const TOSH_TIER_ABI_VERSION: u32 = 1;
pub const TOSH_TIER_STATUS_OK: i32 = 0;
pub const TOSH_TIER_STATUS_INVALID_ARGUMENT: i32 = 1;
pub const TOSH_TIER_STATUS_ABI_MISMATCH: i32 = 2;
pub const TOSH_TIER_STATUS_STRUCT_SIZE_MISMATCH: i32 = 3;

const TOSH_TIER_VERSION: &[u8] = b"0.1.0-phase1\0";

/// Versioned configuration passed across the C ABI.
///
/// Phase 1 intentionally contains no placement knobs. `flags` and `reserved`
/// are zeroed extension points so the ABI can reject incompatible callers
/// before later phases add policy.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ToshTierConfigV1 {
    pub abi_version: u32,
    pub struct_size: u32,
    pub flags: u32,
    pub reserved: u32,
}

/// Opaque runtime handle. C/C++ only ever sees a pointer to this type.
pub struct ToshTierRuntime {
    enabled: bool,
}

fn disabled_by_environment() -> bool {
    std::env::var("TOSH_TIER_DISABLE").is_ok_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

#[no_mangle]
pub extern "C" fn tosh_tier_abi_version() -> u32 {
    TOSH_TIER_ABI_VERSION
}

#[no_mangle]
pub extern "C" fn tosh_tier_version_string() -> *const c_char {
    TOSH_TIER_VERSION.as_ptr().cast()
}

/// Creates an inert Phase 1 runtime and transfers ownership to the caller.
///
/// # Safety
///
/// `config` must point to a readable `ToshTierConfigV1`. `out_runtime` must
/// point to writable storage for one runtime pointer. On success, the caller
/// owns the returned handle and must pass it exactly once to
/// `tosh_tier_runtime_destroy`.
#[no_mangle]
pub unsafe extern "C" fn tosh_tier_runtime_create(
    config: *const ToshTierConfigV1,
    out_runtime: *mut *mut ToshTierRuntime,
) -> i32 {
    if config.is_null() || out_runtime.is_null() {
        return TOSH_TIER_STATUS_INVALID_ARGUMENT;
    }

    // Make failure paths deterministic for C/C++ callers.
    unsafe { ptr::write(out_runtime, ptr::null_mut()) };

    let config = unsafe { &*config };
    if config.abi_version != TOSH_TIER_ABI_VERSION {
        return TOSH_TIER_STATUS_ABI_MISMATCH;
    }
    if usize::from(config.struct_size) != std::mem::size_of::<ToshTierConfigV1>() {
        return TOSH_TIER_STATUS_STRUCT_SIZE_MISMATCH;
    }

    let runtime = Box::new(ToshTierRuntime {
        enabled: !disabled_by_environment(),
    });
    unsafe { ptr::write(out_runtime, Box::into_raw(runtime)) };
    TOSH_TIER_STATUS_OK
}

/// Returns whether the runtime is enabled. A null handle is treated as disabled.
///
/// # Safety
///
/// A non-null `runtime` must be a live handle returned by
/// `tosh_tier_runtime_create` and not yet destroyed.
#[no_mangle]
pub unsafe extern "C" fn tosh_tier_runtime_is_enabled(runtime: *const ToshTierRuntime) -> bool {
    if runtime.is_null() {
        return false;
    }
    unsafe { (*runtime).enabled }
}

/// Destroys a runtime created by `tosh_tier_runtime_create`. Null is a no-op.
///
/// # Safety
///
/// A non-null `runtime` must be a live handle returned by
/// `tosh_tier_runtime_create`, and each handle must be destroyed at most once.
#[no_mangle]
pub unsafe extern "C" fn tosh_tier_runtime_destroy(runtime: *mut ToshTierRuntime) {
    if runtime.is_null() {
        return;
    }
    drop(unsafe { Box::from_raw(runtime) });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_config() -> ToshTierConfigV1 {
        ToshTierConfigV1 {
            abi_version: TOSH_TIER_ABI_VERSION,
            struct_size: u32::try_from(std::mem::size_of::<ToshTierConfigV1>()).unwrap(),
            flags: 0,
            reserved: 0,
        }
    }

    #[test]
    fn config_layout_is_stable() {
        assert_eq!(std::mem::size_of::<ToshTierConfigV1>(), 16);
        assert_eq!(std::mem::align_of::<ToshTierConfigV1>(), 4);
    }

    #[test]
    fn lifecycle_round_trip() {
        let config = valid_config();
        let mut runtime = ptr::null_mut();
        let status = unsafe { tosh_tier_runtime_create(&config, &mut runtime) };
        assert_eq!(status, TOSH_TIER_STATUS_OK);
        assert!(!runtime.is_null());
        unsafe { tosh_tier_runtime_destroy(runtime) };
    }

    #[test]
    fn rejects_wrong_abi() {
        let mut config = valid_config();
        config.abi_version += 1;
        let mut runtime = ptr::null_mut();
        let status = unsafe { tosh_tier_runtime_create(&config, &mut runtime) };
        assert_eq!(status, TOSH_TIER_STATUS_ABI_MISMATCH);
        assert!(runtime.is_null());
    }

    #[test]
    fn rejects_wrong_struct_size() {
        let mut config = valid_config();
        config.struct_size = 0;
        let mut runtime = ptr::null_mut();
        let status = unsafe { tosh_tier_runtime_create(&config, &mut runtime) };
        assert_eq!(status, TOSH_TIER_STATUS_STRUCT_SIZE_MISMATCH);
        assert!(runtime.is_null());
    }
}
