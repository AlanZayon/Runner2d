#include "app_bootstrap.hpp"

#include "log.hpp"
#include "save_store.hpp"
#include "../assets/asset_fs.hpp"

#include "vertical_shift_abi.h"

#include <memory>
#include <string>

int vs_run_host(
    Platform& platform,
    Renderer& renderer,
    const AppBootstrapConfig& cfg) {
    std::unique_ptr<FileAssetFs> file_fs;
    if (!vs_asset_fs()) {
        file_fs = std::make_unique<FileAssetFs>(cfg.asset_dir ? cfg.asset_dir : "assets");
        vs_set_asset_fs(file_fs.get());
    }

    if (vs_runtime_init() != VS_OK) {
        vs_log("vs_runtime_init failed");
        return 1;
    }
    vs_log(
        "Rust runtime %s (ABI %u.%u)",
        vs_runtime_version(),
        vs_abi_version_major(),
        vs_abi_version_minor());

    std::string sprite_path;
    std::string solid_path;
    if (cfg.asset_dir && cfg.asset_dir[0]) {
        sprite_path = std::string(cfg.asset_dir) + "/demo/sprite.png";
        solid_path = std::string(cfg.asset_dir) + "/demo/white.png";
    } else {
        sprite_path = "demo/sprite.png";
        solid_path = "demo/white.png";
    }

    GameplayScene scene;
    if (!scene.init(
            renderer,
            sprite_path.c_str(),
            solid_path.c_str(),
            cfg.seed,
            cfg.seed_locked)) {
        vs_log(
            "GameplayScene init failed (sprite=%s solid=%s)",
            sprite_path.c_str(),
            solid_path.c_str());
        vs_runtime_shutdown();
        return 1;
    }

    if (!cfg.save_override_dir.empty()) {
        SaveStore::set_process_override_dir(cfg.save_override_dir);
    }

    GameLoopStats stats{};
    int rc = 0;
    if (cfg.cpu_bench_ticks > 0) {
        vs_log("CPU-bench mode: %u ticks (no present)", cfg.cpu_bench_ticks);
        rc = run_cpu_bench(scene, cfg.cpu_bench_ticks, cfg.seed, cfg.seed_locked, stats);
    } else {
        InputState input{};
        GameLoopConfig loop_cfg{};
        loop_cfg.enable_validation = cfg.validation;
        loop_cfg.benchmark_seconds = cfg.benchmark_seconds;
        loop_cfg.debug_overlay = cfg.debug_overlay;
        loop_cfg.quality_profile = cfg.quality_profile;
        loop_cfg.record_replay_path = cfg.record_replay_path;
        loop_cfg.replay_path = cfg.replay_path;
        rc = run_game_loop(platform, renderer, scene, input, loop_cfg, stats);
    }

    scene.shutdown();
    vs_runtime_shutdown();
    if (file_fs) {
        vs_set_asset_fs(nullptr);
    }
    return rc;
}
