#include "debug_overlay.hpp"

#include <cstdio>

void debug_overlay_draw(UiDraw& ui, const DebugOverlayStats& stats, const VsRunState& run) {
    char line[192];
    float y = 8.0f;
    const float lh = 18.0f;
    std::snprintf(
        line,
        sizeof(line),
        "FPS %.0f  frame %.2fms  sim %.3f  ext %.3f  draw %.3f",
        stats.fps,
        stats.frame_ms,
        stats.sim_ms,
        stats.extract_ms,
        stats.draw_ms);
    ui.text(8.0f, y, line, {0.85f, 1.0f, 0.7f, 1.0f}, 0.7f);
    y += lh;
    if (stats.gpu_ms >= 0.0) {
        std::snprintf(line, sizeof(line), "GPU %.3fms", stats.gpu_ms);
        ui.text(8.0f, y, line, {0.7f, 0.9f, 1.0f, 1.0f}, 0.7f);
        y += lh;
    }
    std::snprintf(
        line,
        sizeof(line),
        "sprites %u  draws %u  seed %llu  seg %u/%u  q%u  anim %u",
        stats.sprites,
        stats.draw_calls,
        static_cast<unsigned long long>(stats.seed ? stats.seed : run.seed),
        run.segment_index,
        run.segment_count,
        stats.quality,
        stats.anim_clip);
    ui.text(8.0f, y, line, {0.9f, 0.9f, 0.85f, 1.0f}, 0.7f);
    y += lh;
    std::snprintf(
        line,
        sizeof(line),
        "score %u  coins %u  dist %.0f  state %u  fall %u",
        run.score,
        run.coins,
        run.distance,
        run.player_state,
        run.falling);
    ui.text(8.0f, y, line, {0.85f, 0.85f, 0.9f, 1.0f}, 0.7f);
    if (stats.working_set_mb > 0.0f) {
        y += lh;
        std::snprintf(line, sizeof(line), "WS %.1f MB", stats.working_set_mb);
        ui.text(8.0f, y, line, {0.75f, 0.75f, 0.8f, 1.0f}, 0.65f);
    }
}

void debug_overlay_append_aabbs(
    std::vector<SpriteInstance>& dst,
    const VsDebugAabb* aabbs,
    uint32_t aabb_count,
    TextureHandle white,
    float thickness) {
    if (!aabbs || !white.valid()) {
        return;
    }
    for (uint32_t i = 0; i < aabb_count; ++i) {
        const VsDebugAabb& b = aabbs[i];
        const float x = b.x;
        const float y = b.y;
        const float w = b.w;
        const float h = b.h;
        const float t = thickness;
        const float r = (i == 0) ? 0.2f : 1.0f;
        const float g = (i == 0) ? 1.0f : 0.85f;
        const float bl = (i == 0) ? 0.3f : 0.2f;
        auto push = [&](float sx, float sy, float sw, float sh) {
            SpriteInstance s{};
            s.x = sx;
            s.y = sy;
            s.w = sw;
            s.h = sh;
            s.u0 = 0.0f;
            s.v0 = 0.0f;
            s.u1 = 1.0f;
            s.v1 = 1.0f;
            s.r = r;
            s.g = g;
            s.b = bl;
            s.a = 0.85f;
            s.texture = white;
            dst.push_back(s);
        };
        push(x, y, w, t);
        push(x, y + h - t, w, t);
        push(x, y, t, h);
        push(x + w - t, y, t, h);
    }
}
