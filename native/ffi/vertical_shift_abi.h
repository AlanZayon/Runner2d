/**
 * Vertical Shift — shared C ABI between the C++ host and Rust runtime-bridge.
 *
 * Contracts:
 * - No C++ exceptions cross this boundary.
 * - No Rust panics cross this boundary (caught at the Rust FFI edge).
 * - Prefer batch buffer transfers for per-frame data; avoid per-entity calls.
 * - vs_world_* owns simulation state; host provides input and consumes extract buffers.
 */

#ifndef VERTICAL_SHIFT_ABI_H
#define VERTICAL_SHIFT_ABI_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

#define VS_OK 0
#define VS_ERR_FAIL 1
#define VS_ERR_INVALID_ARG 2
#define VS_ERR_NOT_INITIALIZED 3
#define VS_ERR_ALREADY_INITIALIZED 4
#define VS_ERR_NO_WORLD 5

typedef int32_t VsResult;

typedef struct VsInputState {
    float move_x;
    uint32_t jump_pressed;
    uint32_t jump_held;
    uint32_t dash_pressed;
    uint32_t roll_held;
    uint32_t restart_pressed;
    uint32_t qte_pressed;
} VsInputState;

typedef struct VsSpriteInstance {
    float x, y, w, h;
    float u0, v0, u1, v1;
    float r, g, b, a;
    uint32_t texture_id;
} VsSpriteInstance;

typedef struct VsCamera2D {
    float x, y;
    float zoom;
    float viewport_w, viewport_h;
} VsCamera2D;

typedef struct VsRunState {
    uint32_t alive;
    uint32_t finished;
    uint32_t score;
    uint32_t coins;
    float distance;
    uint32_t player_state;
    uint64_t seed;
    uint32_t segment_index;
    uint32_t segment_count;
    uint32_t ability_mask;
    uint32_t qte_active;
    uint32_t qte_prompt;
    uint32_t falling;
    float falling_time;
} VsRunState;

/** Compact gameplay event for host SFX / camera feedback (ABI 1.6+). */
typedef struct VsGameEvent {
    uint32_t kind;
    float x;
    float y;
    uint32_t data;
} VsGameEvent;

/** Debug AABB (ABI 1.7). */
typedef struct VsDebugAabb {
    float x, y, w, h;
} VsDebugAabb;

enum VsQualityProfile {
    VS_QUALITY_LOW = 0,
    VS_QUALITY_BALANCED = 1,
    VS_QUALITY_HIGH = 2,
    VS_QUALITY_ULTRA = 3
};

enum VsGameEventKind {
    VS_EVENT_JUMP = 1,
    VS_EVENT_LAND = 2,
    VS_EVENT_DASH = 3,
    VS_EVENT_ROLL = 4,
    VS_EVENT_COIN = 5,
    VS_EVENT_DEATH = 6,
    VS_EVENT_ABILITY_ON = 7,
    VS_EVENT_FALL_ENTER = 8,
    VS_EVENT_FALL_EXIT = 9,
    VS_EVENT_QTE_TICK = 10,
    VS_EVENT_DESTROY = 11
};

const char* vs_runtime_version(void);
uint32_t vs_abi_version_major(void);
uint32_t vs_abi_version_minor(void);

VsResult vs_runtime_init(void);
uint64_t vs_runtime_tick(void);
VsResult vs_runtime_shutdown(void);

VsResult vs_world_create(void);
VsResult vs_world_create_seeded(uint64_t seed);
VsResult vs_world_destroy(void);
VsResult vs_world_reset(void);
VsResult vs_world_set_viewport(float width, float height);
VsResult vs_world_tick(const VsInputState* input, float dt);
VsResult vs_world_extract(
    VsSpriteInstance* out,
    uint32_t cap,
    uint32_t* out_count,
    VsCamera2D* camera);
VsResult vs_world_get_run_state(VsRunState* out);
/** Drain sim event ring into host buffer; empties drained slots. */
VsResult vs_world_drain_events(VsGameEvent* out, uint32_t cap, uint32_t* count);

/** Phase 12: quality profile (effect caps). */
VsResult vs_world_set_quality(uint32_t profile);
/** Phase 12: debug AABBs for overlay draw. */
VsResult vs_world_extract_debug_aabbs(VsDebugAabb* out, uint32_t cap, uint32_t* count);

#ifdef __cplusplus
}
#endif

#endif /* VERTICAL_SHIFT_ABI_H */
