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

uint32_t tosh_tier_abi_version(void);
const char * tosh_tier_version_string(void);

int32_t tosh_tier_runtime_create(
    const ToshTierConfigV1 * config,
    ToshTierRuntime ** out_runtime);

bool tosh_tier_runtime_is_enabled(const ToshTierRuntime * runtime);
void tosh_tier_runtime_destroy(ToshTierRuntime * runtime);

#ifdef __cplusplus
} // extern "C"

static_assert(sizeof(ToshTierConfigV1) == 16, "ToshTierConfigV1 ABI size changed");
static_assert(alignof(ToshTierConfigV1) == 4, "ToshTierConfigV1 ABI alignment changed");
#endif

#endif // TOSH_TIER_H
