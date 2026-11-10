#ifdef __ANDROID__

#include "app_bootstrap.hpp"
#include "log.hpp"
#include "../assets/asset_fs.hpp"
#include "../platform/android/platform_android.hpp"
#include "../renderer/renderer.hpp"

#include <android_native_app_glue.h>

extern "C" void android_main(struct android_app* state) {
    PlatformAndroid platform(state);

    PlatformWindowDesc desc{};
    desc.title_utf8 = "VERTICAL SHIFT";
    desc.width = 1080;
    desc.height = 1920;
    if (!platform.init(desc)) {
        vs_log("Android platform init failed");
        return;
    }

    AndroidAssetFs assets(platform.asset_manager());
    vs_set_asset_fs(&assets);

    Renderer* renderer = create_vulkan_renderer();
    RendererCreateInfo rci{};
    rci.window_handle = platform.native_window_handle();
    rci.width = platform.width();
    rci.height = platform.height();
    rci.enable_validation = false;
    rci.shader_dir = "shaders";
    if (!renderer->init(rci)) {
        vs_log("Android Vulkan init failed");
        delete renderer;
        platform.shutdown();
        return;
    }

    AppBootstrapConfig cfg{};
    cfg.asset_dir = "";
    cfg.shader_dir = "shaders";
    if (state->activity && state->activity->internalDataPath) {
        cfg.save_override_dir = state->activity->internalDataPath;
    }

    vs_log("VERTICAL SHIFT Phase 9 Android host starting");
    const int rc = vs_run_host(platform, *renderer, cfg);
    vs_log("Android host exit=%d", rc);

    renderer->shutdown();
    delete renderer;
    platform.shutdown();
    vs_set_asset_fs(nullptr);
}

#endif
