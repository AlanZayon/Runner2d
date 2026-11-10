#pragma once

#include "../platform/platform.hpp"

#include "vertical_shift_abi.h"

/** Maps touch streams to the same action edges as keyboard InputState (§20). */
class GestureRecognizer {
public:
    void set_thresholds(float min_distance_px, float max_duration_sec) {
        min_distance_ = min_distance_px;
        max_duration_ = max_duration_sec;
    }

    void on_event(const PlatformEvent& e, double now_sec);

    /** Merge gesture edges into vs_input (OR with keyboard edges). */
    void consume_into(VsInputState& io);

    bool ui_confirm_edge() const { return ui_confirm_; }
    bool ui_back_edge() const { return ui_back_; }
    void clear_ui_edges() {
        ui_confirm_ = false;
        ui_back_ = false;
    }

private:
    struct Finger {
        bool active = false;
        float x0 = 0, y0 = 0;
        float x = 0, y = 0;
        double t0 = 0;
    };

    Finger primary_{};
    float min_distance_ = 48.0f;
    float max_duration_ = 0.35f;

    bool jump_ = false;
    bool dash_ = false;
    bool roll_ = false;
    bool qte_ = false;
    bool ui_confirm_ = false;
    bool ui_back_ = false;

    void finish_gesture(double now_sec);
};
