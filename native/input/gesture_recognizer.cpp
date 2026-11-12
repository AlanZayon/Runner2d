#include "gesture_recognizer.hpp"

#include <cmath>

void GestureRecognizer::on_event(const PlatformEvent& e, double now_sec) {
    switch (e.kind) {
    case PlatformEvent::Kind::Back:
        ui_back_ = true;
        break;
    case PlatformEvent::Kind::TouchDown:
        if (!primary_.active) {
            primary_.active = true;
            primary_.committed = false;
            primary_.id = e.touch_id;
            primary_.x0 = primary_.x = e.x;
            primary_.y0 = primary_.y = e.y;
            primary_.t0 = now_sec;
        }
        break;
    case PlatformEvent::Kind::TouchMove:
        if (primary_.active && e.touch_id == primary_.id) {
            primary_.x = e.x;
            primary_.y = e.y;
            if (!primary_.committed) {
                try_commit_swipe(now_sec);
            }
        }
        break;
    case PlatformEvent::Kind::TouchUp:
        if (primary_.active && e.touch_id == primary_.id) {
            primary_.x = e.x;
            primary_.y = e.y;
            finish_gesture(now_sec);
            primary_.active = false;
            primary_.committed = false;
        }
        break;
    case PlatformEvent::Kind::TouchCancel:
        primary_.active = false;
        primary_.committed = false;
        break;
    default:
        break;
    }
}

bool GestureRecognizer::classify_swipe(float dx, float dy) {
    const float ax = std::fabs(dx);
    const float ay = std::fabs(dy);
    if (ay >= ax * kAxisRatio) {
        if (dy < 0.0f) {
            jump_ = true;
            return true;
        }
        roll_ = true;
        return true;
    }
    if (ax >= ay * kAxisRatio && dx > 0.0f) {
        dash_ = true;
        return true;
    }
    // Ambiguous diagonal or swipe-left — ignore.
    return false;
}

void GestureRecognizer::try_commit_swipe(double now_sec) {
    const float dx = primary_.x - primary_.x0;
    const float dy = primary_.y - primary_.y0;
    const float dist = std::sqrt(dx * dx + dy * dy);
    const float dur = static_cast<float>(now_sec - primary_.t0);

    if (dist < min_distance_ || dur > max_duration_) {
        return;
    }

    if (!classify_swipe(dx, dy)) {
        return;
    }
    if (jump_) {
        jump_hold_until_ = now_sec + kJumpHoldSeconds;
    }
    primary_.committed = true;
}

void GestureRecognizer::finish_gesture(double now_sec) {
    if (primary_.committed) {
        return;
    }

    const float dx = primary_.x - primary_.x0;
    const float dy = primary_.y - primary_.y0;
    const float dist = std::sqrt(dx * dx + dy * dy);
    const float dur = static_cast<float>(now_sec - primary_.t0);

    if (dur > max_duration_ && dist < min_distance_) {
        // Long press without travel — ignore for Phase 9 actions.
        return;
    }

    if (dist < min_distance_) {
        // Tap → UI confirm (Title / GameOver). QTE is filtered out while Playing in game_loop.
        qte_ = true;
        ui_confirm_ = true;
        return;
    }

    if (dur > max_duration_) {
        return;
    }

    if (!classify_swipe(dx, dy)) {
        return;
    }
    if (jump_) {
        jump_hold_until_ = now_sec + kJumpHoldSeconds;
    }
    primary_.committed = true;
}

void GestureRecognizer::consume_into(VsInputState& io, double now_sec) {
    if (jump_) {
        io.jump_pressed = 1;
        io.jump_held = 1;
    }
    if (jump_hold_until_ > now_sec) {
        io.jump_held = 1;
    } else {
        jump_hold_until_ = 0.0;
    }
    if (dash_) {
        io.dash_pressed = 1;
    }
    if (roll_) {
        io.roll_held = 1;
    }
    if (qte_) {
        io.qte_pressed = 1;
    }
    jump_ = dash_ = roll_ = qte_ = false;
}
