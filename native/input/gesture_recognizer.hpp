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
    void consume_into(VsInputState& io, double now_sec);

    bool ui_confirm_edge() const { return ui_confirm_; }
    bool ui_back_edge() const { return ui_back_; }
    void clear_ui_edges() {
        ui_confirm_ = false;
        ui_back_ = false;
    }

private:
    struct Finger {
        bool active = false;
        bool committed = false;
        int id = 0;
        float x0 = 0, y0 = 0;
        float x = 0, y = 0;
        double t0 = 0;
    };

    Finger primary_{};
    float min_distance_ = 24.0f;
    float max_duration_ = 0.65f;

    bool jump_ = false;
    bool dash_ = false;
    bool roll_ = false;
    bool qte_ = false;
    bool ui_confirm_ = false;
    bool ui_back_ = false;

    /// Synthetic hold so a flick reaches near full apex (~JUMP_VELOCITY / GRAVITY).
    double jump_hold_until_ = 0.0;
    static constexpr double kJumpHoldSeconds = 0.38;
    static constexpr float kAxisRatio = 1.25f;

    void try_commit_swipe(double now_sec);
    void finish_gesture(double now_sec);
    bool classify_swipe(float dx, float dy);
};
