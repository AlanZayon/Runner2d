#include "app_bootstrap.hpp"
#include "log.hpp"

#include "../assets/asset_fs.hpp"
#include "../platform/platform.hpp"
#include "../renderer/renderer.hpp"

#include "vertical_shift_abi.h"

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
#include <psapi.h>

#include <cstdlib>
#include <cstdio>
#include <cstring>
#include <string>

#ifndef VS_SHADER_DIR
#define VS_SHADER_DIR "shaders"
#endif
#ifndef VS_ASSET_DIR
#define VS_ASSET_DIR "assets"
#endif

namespace {

bool has_flag(int argc, char** argv, const char* flag) {
    for (int i = 1; i < argc; ++i) {
        if (std::strcmp(argv[i], flag) == 0) {
            return true;
        }
    }
    return false;
}

double flag_double(int argc, char** argv, const char* flag, double fallback) {
    for (int i = 1; i + 1 < argc; ++i) {
        if (std::strcmp(argv[i], flag) == 0) {
            return std::atof(argv[i + 1]);
        }
    }
    return fallback;
}

bool flag_u64_opt(int argc, char** argv, const char* flag, uint64_t* out) {
    for (int i = 1; i + 1 < argc; ++i) {
        if (std::strcmp(argv[i], flag) == 0) {
            *out = static_cast<uint64_t>(std::strtoull(argv[i + 1], nullptr, 0));
            return true;
        }
    }
    return false;
}

uint32_t flag_u32(int argc, char** argv, const char* flag, uint32_t fallback) {
    for (int i = 1; i + 1 < argc; ++i) {
        if (std::strcmp(argv[i], flag) == 0) {
            return static_cast<uint32_t>(std::strtoul(argv[i + 1], nullptr, 0));
        }
    }
    return fallback;
}

const char* flag_str(int argc, char** argv, const char* flag) {
    for (int i = 1; i + 1 < argc; ++i) {
        if (std::strcmp(argv[i], flag) == 0) {
            return argv[i + 1];
        }
    }
    return nullptr;
}

bool dir_exists(const std::string& path) {
    const DWORD attrs = GetFileAttributesA(path.c_str());
    return attrs != INVALID_FILE_ATTRIBUTES && (attrs & FILE_ATTRIBUTE_DIRECTORY);
}

void resolve_runtime_dirs(std::string& asset_dir, std::string& shader_dir) {
    char module_path[MAX_PATH]{};
    GetModuleFileNameA(nullptr, module_path, MAX_PATH);
    std::string exe_dir(module_path);
    const auto slash = exe_dir.find_last_of("\\/");
    if (slash != std::string::npos) {
        exe_dir.resize(slash);
    }

    const std::string local_assets = exe_dir + "\\assets";
    const std::string local_shaders = exe_dir + "\\shaders";
    if (dir_exists(local_assets)) {
        asset_dir = local_assets;
    } else {
        asset_dir = VS_ASSET_DIR;
    }
    if (dir_exists(local_shaders)) {
        shader_dir = local_shaders;
    } else {
        shader_dir = VS_SHADER_DIR;
    }
}

} // namespace

int main(int argc, char** argv) {
    const bool headless_bench = has_flag(argc, argv, "--bench");
    const double bench_seconds = flag_double(argc, argv, "--bench-seconds", 10.0);
    const bool validation = has_flag(argc, argv, "--validation");
    const uint32_t cpu_bench_ticks = flag_u32(argc, argv, "--cpu-bench", 0);
    uint64_t seed = 0;
    const bool seed_locked = flag_u64_opt(argc, argv, "--seed", &seed);
    const bool debug_overlay = has_flag(argc, argv, "--debug-overlay");
    const uint32_t quality = flag_u32(argc, argv, "--quality", 1);
    const char* record_replay = flag_str(argc, argv, "--record-replay");
    const char* replay_path = flag_str(argc, argv, "--replay");

    vs_log("VERTICAL SHIFT Phase 12 host starting");

    std::string asset_dir;
    std::string shader_dir;
    resolve_runtime_dirs(asset_dir, shader_dir);
    vs_log("paths assets=%s shaders=%s", asset_dir.c_str(), shader_dir.c_str());

    Platform* platform = create_platform();
    PlatformWindowDesc desc{};
    desc.title_utf8 = "VERTICAL SHIFT — Phase 12 (spec compliance)";
    desc.width = 432;
    desc.height = 768;
    if (!platform->init(desc)) {
        vs_log("Platform init failed");
        delete platform;
        return 1;
    }

    Renderer* renderer = create_vulkan_renderer();
    RendererCreateInfo rci{};
    rci.window_handle = platform->native_window_handle();
    rci.module_handle = platform->native_module_handle();
    rci.width = platform->width();
    rci.height = platform->height();
    rci.enable_validation = validation;
    rci.shader_dir = shader_dir.c_str();
    rci.asset_dir = asset_dir.c_str();
    if (!renderer->init(rci)) {
        vs_log("Renderer init failed");
        platform->shutdown();
        delete renderer;
        delete platform;
        return 1;
    }

    FileAssetFs assets(asset_dir);
    vs_set_asset_fs(&assets);

    AppBootstrapConfig cfg{};
    cfg.seed = seed;
    cfg.seed_locked = seed_locked;
    cfg.validation = validation;
    cfg.asset_dir = asset_dir.c_str();
    cfg.shader_dir = shader_dir.c_str();
    cfg.cpu_bench_ticks = cpu_bench_ticks;
    cfg.debug_overlay = debug_overlay;
    cfg.quality_profile = quality > 3 ? 1 : quality;
    cfg.record_replay_path = record_replay;
    cfg.replay_path = replay_path;
    if (headless_bench && cpu_bench_ticks == 0) {
        cfg.benchmark_seconds = bench_seconds;
        vs_log("BM mode: %.1fs (idle input)", bench_seconds);
    }

    const int rc = vs_run_host(*platform, *renderer, cfg);

    PROCESS_MEMORY_COUNTERS_EX mem{};
    const bool have_mem = GetProcessMemoryInfo(
                              GetCurrentProcess(),
                              reinterpret_cast<PROCESS_MEMORY_COUNTERS*>(&mem),
                              sizeof(mem)) != 0;
    const double working_set_mb =
        have_mem ? static_cast<double>(mem.WorkingSetSize) / (1024.0 * 1024.0) : -1.0;

    std::printf("\n=== PHASE12 RESULTS ===\n");
    std::printf("seed_locked=%s\n", seed_locked ? "yes" : "no");
    if (seed_locked) {
        std::printf("seed=%llu\n", static_cast<unsigned long long>(seed));
    }
    std::printf("cpu_bench_ticks=%u\n", cpu_bench_ticks);
    std::printf("working_set_mb=%.2f\n", working_set_mb);
    std::printf("validation=%s\n", validation ? "on" : "off");
    std::printf("abi=%u.%u\n", vs_abi_version_major(), vs_abi_version_minor());
    std::printf("=======================\n");

    renderer->shutdown();
    platform->shutdown();
    vs_set_asset_fs(nullptr);
    delete renderer;
    delete platform;
    return rc;
}
