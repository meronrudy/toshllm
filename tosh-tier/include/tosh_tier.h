#ifndef TOSH_TIER_H
#define TOSH_TIER_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define TOSH_TIER_ABI_VERSION 1u

#define TOSH_TIER_STATUS_OK 0
#define TOSH_TIER_STATUS_INVALID_ARGUMENT 1
#define TOSH_TIER_STATUS_ABI_MISMATCH 2
#define TOSH_TIER_STATUS_STRUCT_SIZE_MISMATCH 3

typedef struct ToshTierRuntime ToshTierRuntime;

typedef struct ToshTierConfigV1 {
    uint32_t abi_version;
    uint32_t struct_size;
    uint32_t flags;
    uint32_t reserved;
} ToshTierConfigV1;

typedef struct ToshTierRuntimeStatsV1 {
    uint32_t abi_version;
    uint32_t struct_size;
    uint64_t router_events;
    uint64_t selected_experts;
    uint64_t vram_samples;
    uint64_t transfers_submitted;
    uint64_t transfers_completed;
    uint64_t cache_hits;
    uint64_t cache_misses;
} ToshTierRuntimeStatsV1;

uint32_t tosh_tier_abi_version(void);
const char * tosh_tier_version_string(void);

int32_t tosh_tier_runtime_create(
    const ToshTierConfigV1 * config,
    ToshTierRuntime ** out_runtime);

bool tosh_tier_runtime_is_enabled(const ToshTierRuntime * runtime);

int32_t tosh_tier_record_router_selection(
    const ToshTierRuntime * runtime,
    uint64_t sequence_id,
    uint64_t token_position,
    uint32_t layer,
    const uint32_t * expert_ids,
    uint32_t expert_count);

int32_t tosh_tier_record_vram_sample(
    const ToshTierRuntime * runtime,
    uint16_t gpu_index,
    uint64_t used_bytes,
    uint64_t free_bytes);

int32_t tosh_tier_runtime_stats(
    const ToshTierRuntime * runtime,
    ToshTierRuntimeStatsV1 * out_stats);

void tosh_tier_runtime_destroy(ToshTierRuntime * runtime);

#ifdef __cplusplus
} // extern "C"

static_assert(sizeof(ToshTierConfigV1) == 16, "ToshTierConfigV1 ABI size changed");
static_assert(alignof(ToshTierConfigV1) == 4, "ToshTierConfigV1 ABI alignment changed");
static_assert(sizeof(ToshTierRuntimeStatsV1) == 64, "ToshTierRuntimeStatsV1 ABI size changed");
static_assert(alignof(ToshTierRuntimeStatsV1) == 8, "ToshTierRuntimeStatsV1 ABI alignment changed");
#endif

#endif // TOSH_TIER_H
