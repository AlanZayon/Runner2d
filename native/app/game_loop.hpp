#pragma once

#include "../platform/platform.hpp"
#include "../renderer/renderer.hpp"
#include "gameplay_scene.hpp"
#include "input_state.hpp"

// input_state.hpp includes vertical_shift_abi.h (VsInputState).

struct GameLoopConfig {
    double fixed_dt = 1.0 / 60.0;
    int max_updates_per_frame = 5;
    double benchmark_seconds = 0.0;
    bool enable_validation = false;
    /** Start with F3 overlay visible (also toggleable at runtime). Off for Release pack. */
    bool debug_overlay = false;
    uint32_t quality_profile = 1; // VsQualityProfile
    const char* record_replay_path = nullptr;
    const char* replay_path = nullptr;
};

struct GameLoopStats {
    uint64_t frames = 0;
    uint64_t sim_ticks = 0;
    double elapsed_seconds = 0.0;
    double sum_frame_ms = 0.0;
    double max_frame_ms = 0.0;
    double sum_sim_ms = 0.0;
    double sum_extract_ms = 0.0;
    double sum_draw_ms = 0.0;
    uint32_t last_draw_calls = 0;
    uint32_t last_sprites = 0;
    static constexpr int kSamples = 512;
    double recent_ms[kSamples]{};
    int recent_count = 0;
    int recent_index = 0;
};

int run_game_loop(
    Platform& platform,
    Renderer& renderer,
    GameplayScene& scene,
    InputState& input,
    const GameLoopConfig& config,
    GameLoopStats& stats);

/** CPU-unbound sim+extract loop (no present). Returns 0 on success. */
int run_cpu_bench(
    GameplayScene& scene,
    uint32_t ticks,
    uint64_t seed,
    bool seed_locked,
    GameLoopStats& stats);

double vs_stats_approx_p95(const GameLoopStats& stats);
