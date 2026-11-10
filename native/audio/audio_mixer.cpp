#include "audio_mixer.hpp"

#include "log_audio.hpp"
#include "../assets/asset_fs.hpp"
#include "../app/save_store.hpp"

#include "miniaudio.h"

#include <cstdio>
#include <cstring>
#include <fstream>
#include <new>
#include <string>
#include <sys/stat.h>
#include <vector>

#ifdef _WIN32
#include <direct.h>
#define VS_MKDIR(path) _mkdir(path)
#else
#include <sys/types.h>
#define VS_MKDIR(path) mkdir(path, 0755)
#endif

namespace {

const char* kClipNames[static_cast<int>(SfxId::Count)] = {
    "jump.wav",
    "land.wav",
    "dash.wav",
    "roll.wav",
    "coin.wav",
    "death.wav",
    "ability.wav",
    "fall_enter.wav",
    "fall_exit.wav",
    "qte.wav",
    "destroy.wav",
};

} // namespace

bool AudioMixer::init(AudioDevice& device, const char* audio_dir) {
    shutdown();
    device_ = &device;
    if (!device.ready() || !device.engine()) {
        vs_audio_log("mixer: device not ready — mute");
        ready_ = false;
        return false;
    }
    auto* eng = static_cast<ma_engine*>(device.engine());
    char path[512];
    int loaded = 0;
    for (int i = 0; i < kClips; ++i) {
        sounds_[i] = nullptr;
        clip_ok_[i] = false;
        if (!audio_dir || !audio_dir[0]) {
            continue;
        }
        std::snprintf(path, sizeof(path), "%s/%s", audio_dir, kClipNames[i]);
        // On Android (or when file missing), extract via AssetFs to a writable cache.
        if (AssetFs* fs = vs_asset_fs()) {
            std::vector<uint8_t> bytes;
            if (fs->read_bytes(path, bytes) && !bytes.empty()) {
                std::string cache = SaveStore::process_override_dir();
                if (cache.empty()) {
                    cache = ".";
                }
                cache += "/sfx_cache";
                VS_MKDIR(cache.c_str());
                const std::string out_path = cache + "/" + kClipNames[i];
                std::ofstream out(out_path, std::ios::binary);
                if (out) {
                    out.write(reinterpret_cast<const char*>(bytes.data()),
                              static_cast<std::streamsize>(bytes.size()));
                    out.close();
                    std::snprintf(path, sizeof(path), "%s", out_path.c_str());
                }
            }
        }
        auto* snd = new (std::nothrow) ma_sound();
        if (!snd) {
            continue;
        }
        const ma_result r = ma_sound_init_from_file(
            eng,
            path,
            MA_SOUND_FLAG_DECODE | MA_SOUND_FLAG_ASYNC | MA_SOUND_FLAG_NO_PITCH,
            nullptr,
            nullptr,
            snd);
        if (r != MA_SUCCESS) {
            vs_audio_log("mixer: missing/fail %s (%d)", path, static_cast<int>(r));
            delete snd;
            continue;
        }
        ma_sound_set_volume(snd, 0.0f); // template; voices copy volume at play
        sounds_[i] = snd;
        clip_ok_[i] = true;
        ++loaded;
    }
    for (int i = 0; i < kVoices; ++i) {
        voices_[i] = nullptr;
    }
    ready_ = loaded > 0;
    apply_volumes();
    vs_audio_log("mixer: loaded %d/%d clips", loaded, kClips);
    return ready_;
}

void AudioMixer::shutdown() {
    for (int i = 0; i < kVoices; ++i) {
        if (voices_[i]) {
            ma_sound_uninit(static_cast<ma_sound*>(voices_[i]));
            delete static_cast<ma_sound*>(voices_[i]);
            voices_[i] = nullptr;
        }
    }
    for (int i = 0; i < kClips; ++i) {
        if (sounds_[i]) {
            ma_sound_uninit(static_cast<ma_sound*>(sounds_[i]));
            delete static_cast<ma_sound*>(sounds_[i]);
            sounds_[i] = nullptr;
        }
        clip_ok_[i] = false;
    }
    device_ = nullptr;
    ready_ = false;
}

void AudioMixer::set_master_volume(float v) {
    master_ = v < 0.0f ? 0.0f : (v > 1.0f ? 1.0f : v);
    apply_volumes();
}

void AudioMixer::set_sfx_volume(float v) {
    sfx_ = v < 0.0f ? 0.0f : (v > 1.0f ? 1.0f : v);
    apply_volumes();
}

void AudioMixer::apply_volumes() {
    if (!device_ || !device_->ready() || !device_->engine()) {
        return;
    }
    ma_engine_set_volume(static_cast<ma_engine*>(device_->engine()), master_);
}

void AudioMixer::play(SfxId id) {
    if (!ready_ || !device_ || !device_->ready()) {
        return;
    }
    const int idx = static_cast<int>(id);
    if (idx < 0 || idx >= kClips || !clip_ok_[idx] || !sounds_[idx]) {
        return;
    }
    auto* eng = static_cast<ma_engine*>(device_->engine());
    // Steal oldest voice slot.
    if (voices_[next_voice_]) {
        ma_sound_uninit(static_cast<ma_sound*>(voices_[next_voice_]));
        delete static_cast<ma_sound*>(voices_[next_voice_]);
        voices_[next_voice_] = nullptr;
    }
    auto* voice = new (std::nothrow) ma_sound();
    if (!voice) {
        return;
    }
    const ma_result r = ma_sound_init_copy(
        eng,
        static_cast<ma_sound*>(sounds_[idx]),
        0,
        nullptr,
        voice);
    if (r != MA_SUCCESS) {
        delete voice;
        return;
    }
    ma_sound_set_volume(voice, sfx_);
    ma_sound_start(voice);
    voices_[next_voice_] = voice;
    next_voice_ = (next_voice_ + 1) % kVoices;
}

bool AudioMixer::play_event(uint32_t event_kind) {
    SfxId id = SfxId::Count;
    switch (event_kind) {
    case 1: id = SfxId::Jump; break;
    case 2: id = SfxId::Land; break;
    case 3: id = SfxId::Dash; break;
    case 4: id = SfxId::Roll; break;
    case 5: id = SfxId::Coin; break;
    case 6: id = SfxId::Death; break;
    case 7: id = SfxId::Ability; break;
    case 8: id = SfxId::FallEnter; break;
    case 9: id = SfxId::FallExit; break;
    case 10: id = SfxId::Qte; break;
    case 11: id = SfxId::Destroy; break;
    default: return false;
    }
    play(id);
    return true;
}
