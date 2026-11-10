#pragma once

#include "ui_draw.hpp"

#include "vertical_shift_abi.h"

enum class AppScreen {
    Title,
    Playing,
    Paused,
    GameOver
};

struct ScreenDrawParams {
    float viewport_w = 1280.0f;
    float viewport_h = 720.0f;
    const VsRunState* run = nullptr;
    uint32_t best_score = 0;
    float best_distance = 0.0f;
    float vol_master = 1.0f;
    float vol_sfx = 1.0f;
    bool touch_ui = false;
};

void screens_draw(UiDraw& ui, AppScreen screen, const ScreenDrawParams& p);
