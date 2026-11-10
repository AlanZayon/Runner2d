#include "audio_device.hpp"

#include "log_audio.hpp"

#define MINIAUDIO_IMPLEMENTATION
#include "miniaudio.h"

#include <new>

bool AudioDevice::init() {
    if (ready_) {
        return true;
    }
    auto* eng = new (std::nothrow) ma_engine();
    if (!eng) {
        vs_audio_log("audio: alloc failed");
        return false;
    }
    ma_engine_config cfg = ma_engine_config_init();
    const ma_result r = ma_engine_init(&cfg, eng);
    if (r != MA_SUCCESS) {
        vs_audio_log("audio: ma_engine_init failed (%d) — SFX muted", static_cast<int>(r));
        delete eng;
        engine_ = nullptr;
        ready_ = false;
        return false;
    }
    engine_ = eng;
    ready_ = true;
    vs_audio_log("audio: engine ready");
    return true;
}

void AudioDevice::shutdown() {
    if (engine_) {
        ma_engine_uninit(static_cast<ma_engine*>(engine_));
        delete static_cast<ma_engine*>(engine_);
        engine_ = nullptr;
    }
    ready_ = false;
}
