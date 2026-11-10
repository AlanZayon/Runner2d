#pragma once

#include "vertical_shift_abi.h"

#include <cstdint>
#include <cstdio>
#include <string>
#include <vector>

/** Binary tick stream of VsInputState (§39). */
class InputReplay {
public:
    bool begin_record(const char* path);
    void record_tick(const VsInputState& input);
    void end_record();

    bool begin_playback(const char* path);
    bool next_tick(VsInputState& out);
    void end_playback();

    bool recording() const { return record_file_ != nullptr; }
    bool playing() const { return !playback_.empty() && play_index_ < playback_.size(); }

private:
    FILE* record_file_ = nullptr;
    std::vector<VsInputState> playback_;
    size_t play_index_ = 0;
};
