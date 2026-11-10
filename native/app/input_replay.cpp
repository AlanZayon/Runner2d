#include "input_replay.hpp"

#include "log.hpp"

#include <cstring>

bool InputReplay::begin_record(const char* path) {
    end_record();
    if (!path || !path[0]) {
        return false;
    }
#ifdef _MSC_VER
    if (fopen_s(&record_file_, path, "wb") != 0) {
        record_file_ = nullptr;
    }
#else
    record_file_ = std::fopen(path, "wb");
#endif
    if (!record_file_) {
        vs_log("replay: cannot open record %s", path);
        return false;
    }
    vs_log("replay: recording → %s", path);
    return true;
}

void InputReplay::record_tick(const VsInputState& input) {
    if (!record_file_) {
        return;
    }
    std::fwrite(&input, sizeof(VsInputState), 1, record_file_);
}

void InputReplay::end_record() {
    if (record_file_) {
        std::fclose(record_file_);
        record_file_ = nullptr;
        vs_log("replay: record closed");
    }
}

bool InputReplay::begin_playback(const char* path) {
    end_playback();
    if (!path || !path[0]) {
        return false;
    }
    FILE* f = nullptr;
#ifdef _MSC_VER
    if (fopen_s(&f, path, "rb") != 0) {
        f = nullptr;
    }
#else
    f = std::fopen(path, "rb");
#endif
    if (!f) {
        vs_log("replay: cannot open playback %s", path);
        return false;
    }
    VsInputState tick{};
    while (std::fread(&tick, sizeof(VsInputState), 1, f) == 1) {
        playback_.push_back(tick);
    }
    std::fclose(f);
    play_index_ = 0;
    vs_log("replay: loaded %zu ticks from %s", playback_.size(), path);
    return !playback_.empty();
}

bool InputReplay::next_tick(VsInputState& out) {
    if (play_index_ >= playback_.size()) {
        std::memset(&out, 0, sizeof(out));
        return false;
    }
    out = playback_[play_index_++];
    return true;
}

void InputReplay::end_playback() {
    playback_.clear();
    play_index_ = 0;
}
