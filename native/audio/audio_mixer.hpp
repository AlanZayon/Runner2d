#pragma once

#include "audio_device.hpp"

#include <cstdint>

enum class SfxId : uint32_t {
    Jump = 0,
    Land,
    Dash,
    Roll,
    Coin,
    Death,
    Ability,
    FallEnter,
    FallExit,
    Qte,
    Destroy,
    Count
};

class AudioMixer {
public:
    bool init(AudioDevice& device, const char* audio_dir);
    void shutdown();

    void set_master_volume(float v);
    void set_sfx_volume(float v);
    float master_volume() const { return master_; }
    float sfx_volume() const { return sfx_; }

    void play(SfxId id);
    /** Map VsGameEventKind → SFX (returns false if unknown). */
    bool play_event(uint32_t event_kind);

private:
    static constexpr int kVoices = 12;
    static constexpr int kClips = static_cast<int>(SfxId::Count);

    AudioDevice* device_ = nullptr;
    void* sounds_[kClips]{};   // ma_sound* decoded clips (templates)
    void* voices_[kVoices]{};  // ma_sound* one-shot voices
    bool clip_ok_[kClips]{};
    int next_voice_ = 0;
    float master_ = 1.0f;
    float sfx_ = 1.0f;
    bool ready_ = false;

    void apply_volumes();
};
