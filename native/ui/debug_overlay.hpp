#pragma once

#include "font_atlas.hpp"
#include "ui_draw.hpp"
#include "vertical_shift_abi.h"

#include <cstdint>
#include <vector>

struct DebugOverlayStats {
    double fps = 0.0;
    double frame_ms = 0.0;
    double sim_ms = 0.0;
    double extract_ms = 0.0;
    double draw_ms = 0.0;
    double gpu_ms = -1.0; // <0 if unavailable
    uint32_t sprites = 0;
    uint32_t draw_calls = 0;
    uint64_t seed = 0;
    uint32_t quality = 1;
    uint32_t anim_clip = 0;
    float working_set_mb = -1.0f;
};

/** F3 overlay text (dev). Appends into `ui` after `ui.begin(...)`. */
void debug_overlay_draw(UiDraw& ui, const DebugOverlayStats& stats, const VsRunState& run);

/** Append thin AABB outlines as tinted sprites into `dst`. */
void debug_overlay_append_aabbs(
    std::vector<SpriteInstance>& dst,
    const VsDebugAabb* aabbs,
    uint32_t aabb_count,
    TextureHandle white,
    float thickness = 2.0f);
