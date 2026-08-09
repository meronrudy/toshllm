#![allow(unsafe_code)]
#![allow(clippy::must_use_candidate)]

use std::os::raw::c_char;
use std::ptr;
use std::sync::atomic::{AtomicU64, Ordering};

/// Stable C ABI version for the experimental ToshLLM tier runtime.
pub const TOSH_TIER_ABI_VERSION: u32 = 1;
pub const TOSH_TIER_STATUS_OK: i32 = 0;
pub const TOSH_TIER_STATUS_INVALID_ARGUMENT: i32 = 1;
pub const TOSH_TIER_STATUS_ABI_MISMATCH: i32 = 2;
pub const TOSH_TIER_STATUS_STRUCT_SIZE_MISMATCH: i32 = 3;

const TOSH_TIER_VERSION: &[u8] = b"0.2.0-scaffold\0";

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ToshTierConfigV1 {
    pub abi_version: u32,
    pub struct_size: u32,
    pub flags: u32,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ToshTierRuntimeStatsV1 {
    pub abi_version: u32,
    pub struct_size: u32,
    pub router_events: u64,
    pub selected_experts: u64,
    pub vram_samples: u64,
    pub transfers_submitted: u64,
    pub transfers_completed: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

/// Opaque runtime handle. Phase 2+ hooks only mutate atomic telemetry here;
/// placement and transfer execution remain disabled until explicitly wired.
pub struct ToshTierRuntime {
    enabled: bool,
    router_events: AtomicU64,
    selected_experts: AtomicU64,
    vram_samples: AtomicU64,
    transfers_submitted: AtomicU64,
    transfers_completed: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
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

/// Creates the runtime and transfers ownership to the caller.
///
/// # Safety
///
/// `config` must point to a readable `ToshTierConfigV1`. `out_runtime` must
/// point to writable storage for one runtime pointer. On success the caller
/// owns the handle and must destroy it exactly once.
#[no_mangle]
pub unsafe extern "C" fn tosh_tier_runtime_create(
    config: *const ToshTierConfigV1,
    out_runtime: *mut *mut ToshTierRuntime,
) -> i32 {
    if config.is_null() || out_runtime.is_null() {
        return TOSH_TIER_STATUS_INVALID_ARGUMENT;
    }

    unsafe { ptr::write(out_runtime, ptr::null_mut()) };

    let config = unsafe { &*config };
    if config.abi_version != TOSH_TIER_ABI_VERSION {
        return TOSH_TIER_STATUS_ABI_MISMATCH;
    }
    if usize::try_from(config.struct_size).ok() != Some(std::mem::size_of::<ToshTierConfigV1>()) {
        return TOSH_TIER_STATUS_STRUCT_SIZE_MISMATCH;
    }

    let runtime = Box::new(ToshTierRuntime {
        enabled: !disabled_by_environment(),
        router_events: AtomicU64::new(0),
        selected_experts: AtomicU64::new(0),
        vram_samples: AtomicU64::new(0),
        transfers_submitted: AtomicU64::new(0),
        transfers_completed: AtomicU64::new(0),
        cache_hits: AtomicU64::new(0),
        cache_misses: AtomicU64::new(0),
    });
    unsafe { ptr::write(out_runtime, Box::into_raw(runtime)) };
    TOSH_TIER_STATUS_OK
}

/// Returns whether the runtime is enabled. A null handle is disabled.
///
/// # Safety
///
/// A non-null `runtime` must be a live handle returned by create.
#[no_mangle]
pub unsafe extern "C" fn tosh_tier_runtime_is_enabled(runtime: *const ToshTierRuntime) -> bool {
    if runtime.is_null() {
        return false;
    }
    unsafe { (*runtime).enabled }
}

/// Records a read-only MoE router observation for Phase 2 instrumentation.
///
/// # Safety
///
/// `runtime` must be a live runtime. If `expert_count` is non-zero,
/// `expert_ids` must be a readable array with at least that many entries.
#[no_mangle]
pub unsafe extern "C" fn tosh_tier_record_router_selection(
    runtime: *const ToshTierRuntime,
    _sequence_id: u64,
    _token_position: u64,
    _layer: u32,
    expert_ids: *const u32,
    expert_count: u32,
) -> i32 {
    if runtime.is_null() || (expert_count > 0 && expert_ids.is_null()) {
        return TOSH_TIER_STATUS_INVALID_ARGUMENT;
    }
    let runtime = unsafe { &*runtime };
    if runtime.enabled {
        runtime.router_events.fetch_add(1, Ordering::Relaxed);
        runtime
            .selected_experts
            .fetch_add(u64::from(expert_count), Ordering::Relaxed);
    }
    TOSH_TIER_STATUS_OK
}

/// Records VRAM pressure telemetry without synchronizing the GPU.
///
/// # Safety
///
/// `runtime` must be a live runtime.
#[no_mangle]
pub unsafe extern "C" fn tosh_tier_record_vram_sample(
    runtime: *const ToshTierRuntime,
    _gpu_index: u16,
    _used_bytes: u64,
    _free_bytes: u64,
) -> i32 {
    if runtime.is_null() {
        return TOSH_TIER_STATUS_INVALID_ARGUMENT;
    }
    let runtime = unsafe { &*runtime };
    if runtime.enabled {
        runtime.vram_samples.fetch_add(1, Ordering::Relaxed);
    }
    TOSH_TIER_STATUS_OK
}

/// Copies an atomic runtime telemetry snapshot into caller-owned storage.
///
/// # Safety
///
/// `runtime` must be live and `out_stats` must point to writable storage for
/// one `ToshTierRuntimeStatsV1`.
#[no_mangle]
pub unsafe extern "C" fn tosh_tier_runtime_stats(
    runtime: *const ToshTierRuntime,
    out_stats: *mut ToshTierRuntimeStatsV1,
) -> i32 {
    if runtime.is_null() || out_stats.is_null() {
        return TOSH_TIER_STATUS_INVALID_ARGUMENT;
    }
    let runtime = unsafe { &*runtime };
    let stats = ToshTierRuntimeStatsV1 {
        abi_version: TOSH_TIER_ABI_VERSION,
        struct_size: u32::try_from(std::mem::size_of::<ToshTierRuntimeStatsV1>())
            .unwrap_or_default(),
        router_events: runtime.router_events.load(Ordering::Relaxed),
        selected_experts: runtime.selected_experts.load(Ordering::Relaxed),
        vram_samples: runtime.vram_samples.load(Ordering::Relaxed),
        transfers_submitted: runtime.transfers_submitted.load(Ordering::Relaxed),
        transfers_completed: runtime.transfers_completed.load(Ordering::Relaxed),
        cache_hits: runtime.cache_hits.load(Ordering::Relaxed),
        cache_misses: runtime.cache_misses.load(Ordering::Relaxed),
    };
    unsafe { ptr::write(out_stats, stats) };
    TOSH_TIER_STATUS_OK
}

/// Destroys a runtime created by `tosh_tier_runtime_create`. Null is a no-op.
///
/// # Safety
///
/// A non-null `runtime` must be a live handle returned by create and each
/// handle must be destroyed at most once.
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
    fn abi_layouts_are_stable() {
        assert_eq!(std::mem::size_of::<ToshTierConfigV1>(), 16);
        assert_eq!(std::mem::align_of::<ToshTierConfigV1>(), 4);
        assert_eq!(std::mem::size_of::<ToshTierRuntimeStatsV1>(), 64);
        assert_eq!(std::mem::align_of::<ToshTierRuntimeStatsV1>(), 8);
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
