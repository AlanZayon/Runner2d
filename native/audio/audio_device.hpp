#pragma once

/** Thin miniaudio device + engine bootstrap (Phase 8). Soft-fails if no device. */
struct AudioDevice {
    bool init();
    void shutdown();
    bool ready() const { return ready_; }

    void* engine() { return engine_; }

private:
    bool ready_ = false;
    void* engine_ = nullptr; // ma_engine*
};
