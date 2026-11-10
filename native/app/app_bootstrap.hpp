#pragma once

#include "../platform/platform.hpp"
#include "../renderer/renderer.hpp"
#include "game_loop.hpp"
#include "gameplay_scene.hpp"
#include "input_state.hpp"

#include <string>

struct AppBootstrapConfig {
    uint64_t seed = 0;
    bool seed_locked = false;
    bool validation = false;
    double benchmark_seconds = 0.0;
    uint32_t cpu_bench_ticks = 0; // >0 → CPU-unbound sim+extract, no present loop
    const char* asset_dir = "assets";
    const char* shader_dir = "shaders";
    std::string save_override_dir; // Android filesDir
    bool debug_overlay = false;
    uint32_t quality_profile = 1;
    const char* record_replay_path = nullptr;
    const char* replay_path = nullptr;
};

/** Shared host boot after Platform + Renderer exist. Returns process exit code. */
int vs_run_host(
    Platform& platform,
    Renderer& renderer,
    const AppBootstrapConfig& cfg);
